use std::sync::{Arc, Weak};

use kas_l2_atomic::{AtomicEnum, AtomicWeak};

use crate::{access_type::AccessType, consumer::Consumer};

pub struct ResourceGuard<N: Consumer> {
    pub status: AtomicEnum<Status>,
    pub access_type: AtomicEnum<AccessType>,
    pub successor: AtomicWeak<ResourceGuard<N>>,
    pub notifier: AtomicWeak<N>,
}

impl<N: Consumer> ResourceGuard<N> {
    pub fn new(notifier: Weak<N>, access_type: AccessType) -> Self {
        Self {
            access_type: AtomicEnum::new(access_type),
            status: AtomicEnum::new(Status::Waiting),
            successor: AtomicWeak::default(),
            notifier: AtomicWeak::new(notifier),
        }
    }

    pub fn extend(&self, successor: &Arc<ResourceGuard<N>>) {
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

    pub fn ready(&self) {
        if self
            .status
            .compare_exchange(Status::Waiting, Status::Ready)
            .is_ok()
        {
            if let Some(owner) = self.notifier.load().upgrade() {
                owner.notify_ready();
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

pub type PrevGuards<E> = Vec<Option<Arc<ResourceGuard<E>>>>;

pub type NewGuards<E> = Vec<Arc<ResourceGuard<E>>>;

pub type Guards<E> = (PrevGuards<E>, NewGuards<E>);

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
