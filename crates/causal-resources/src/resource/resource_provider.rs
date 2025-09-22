use std::sync::{Arc, Weak};

use kas_l2_atomic::{AtomicEnum, AtomicOptionArc, AtomicWeak};

use crate::resource::{
    access_status::AccessStatus, access_type::AccessType, resource_consumer::ResourceConsumer,
};

pub struct ResourceProvider<C: ResourceConsumer> {
    status: AtomicEnum<AccessStatus>,
    pub access_type: AtomicEnum<AccessType>,
    pub consumer: (AtomicWeak<C>, C::ResourceID),
    pub prev: AtomicOptionArc<Self>,
    pub next: AtomicWeak<Self>,
}

impl<C: ResourceConsumer> ResourceProvider<C> {
    pub fn new(
        prev: Option<Arc<Self>>,
        consumer: Weak<C>,
        consumer_guard_id: C::ResourceID,
        access_type: AccessType,
    ) -> Self {
        Self {
            access_type: AtomicEnum::new(access_type),
            status: AtomicEnum::new(AccessStatus::Waiting),
            consumer: (AtomicWeak::new(consumer), consumer_guard_id),
            prev: AtomicOptionArc::new(prev),
            next: AtomicWeak::default(),
        }
    }

    pub fn extend(&self, successor: &Arc<Self>) {
        self.next.store(Arc::downgrade(successor));

        match self.status.load() {
            AccessStatus::Ready if self.access_type.load() == AccessType::Read => {
                if successor.access_type.load() == AccessType::Read {
                    successor.ready();
                }
            }
            AccessStatus::Done => successor.ready(),
            _ => {} // do nothing, the successor will be notified when anything changes
        }
    }

    pub fn ready(self: &Arc<Self>) {
        if self
            .status
            .compare_exchange(AccessStatus::Waiting, AccessStatus::Ready)
            .is_ok()
        {
            if let Some(owner) = self.consumer.0.load().upgrade() {
                owner.notify(self.clone());
            } else {
                eprintln!("ResourceGuard::ready: notifier is gone");
            }

            if self.access_type.load() == AccessType::Read {
                if let Some(successor) = self.next.load().upgrade() {
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
            .compare_exchange(AccessStatus::Ready, AccessStatus::Done)
            .is_ok()
        {
            if let Some(successor) = self.next.load().upgrade() {
                successor.ready();
            }
        }
    }
}
