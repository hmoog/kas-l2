use std::sync::{Arc, Weak};

use kas_l2_atomic::{AtomicEnum, AtomicOptionArc, AtomicWeak};

use crate::{access_type::AccessType, resource_consumer::GuardConsumer};

pub struct ResourceAccess<C: GuardConsumer> {
    pub status: AtomicEnum<Status>,
    pub access_type: AtomicEnum<AccessType>,
    pub prev: AtomicOptionArc<ResourceAccess<C>>,
    pub successor: AtomicWeak<ResourceAccess<C>>,
    pub consumer: AtomicWeak<C>,
    pub consumer_id: C::ConsumerGuardID,
}

impl<C: GuardConsumer> ResourceAccess<C> {
    pub fn new(
        prev: Option<Arc<ResourceAccess<C>>>,
        consumer: Weak<C>,
        consumer_guard_id: C::ConsumerGuardID,
        access_type: AccessType,
    ) -> Self {
        Self {
            prev: AtomicOptionArc::new(prev),
            access_type: AtomicEnum::new(access_type),
            status: AtomicEnum::new(Status::Waiting),
            successor: AtomicWeak::default(),
            consumer: AtomicWeak::new(consumer),
            consumer_id: consumer_guard_id,
        }
    }

    pub fn extend(&self, successor: &Arc<ResourceAccess<C>>) {
        self.successor.store(Arc::downgrade(successor));

        match self.status.load() {
            Status::Ready if self.access_type.load() == AccessType::Read => {
                if successor.access_type.load() == AccessType::Read {
                    successor.ready();
                }
            }
            Status::Done => successor.ready(),
            _ => {} // do nothing, the successor will be notified when anything changes
        }
    }

    pub fn ready(self: &Arc<Self>) {
        if self
            .status
            .compare_exchange(Status::Waiting, Status::Ready)
            .is_ok()
        {
            if let Some(owner) = self.consumer.load().upgrade() {
                owner.notify(self.clone());
            } else {
                eprintln!("ResourceGuard::ready: notifier is gone");
            }

            if self.access_type.load() == AccessType::Read {
                if let Some(successor) = self.successor.load().upgrade() {
                    if successor.access_type.load() == AccessType::Read {
                        successor.ready();
                    }
                }
            }
        }
    }

    pub fn done(&self) {
        if self
            .status
            .compare_exchange(Status::Ready, Status::Done)
            .is_ok()
        {
            if let Some(successor) = self.successor.load().upgrade() {
                successor.ready();
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Waiting = 0,
    Ready = 1,
    Done = 2,
}

mod traits {
    use super::Status;

    impl From<Status> for u8 {
        fn from(s: Status) -> Self {
            s as u8
        }
    }

    impl TryFrom<u8> for Status {
        type Error = ();

        fn try_from(v: u8) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(Status::Waiting),
                1 => Ok(Status::Ready),
                2 => Ok(Status::Done),
                _ => Err(()),
            }
        }
    }
}
