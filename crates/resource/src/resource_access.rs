use std::sync::{Arc, Weak};

use kas_l2_atomic::{AtomicEnum, AtomicOptionArc, AtomicWeak};

use crate::{AccessType, ResourceConsumer, ResourceStatus};

pub struct ResourceAccess<C: ResourceConsumer> {
    pub status: AtomicEnum<ResourceStatus>,
    pub access_type: AtomicEnum<AccessType>,
    pub prev: AtomicOptionArc<ResourceAccess<C>>,
    pub successor: AtomicWeak<ResourceAccess<C>>,
    pub consumer: AtomicWeak<C>,
    pub consumer_id: C::ResourceID,
}

impl<C: ResourceConsumer> ResourceAccess<C> {
    pub fn new(
        prev: Option<Arc<ResourceAccess<C>>>,
        consumer: Weak<C>,
        consumer_guard_id: C::ResourceID,
        access_type: AccessType,
    ) -> Self {
        Self {
            prev: AtomicOptionArc::new(prev),
            access_type: AtomicEnum::new(access_type),
            status: AtomicEnum::new(ResourceStatus::Waiting),
            successor: AtomicWeak::default(),
            consumer: AtomicWeak::new(consumer),
            consumer_id: consumer_guard_id,
        }
    }

    pub fn extend(&self, successor: &Arc<ResourceAccess<C>>) {
        self.successor.store(Arc::downgrade(successor));

        match self.status.load() {
            ResourceStatus::Ready if self.access_type.load() == AccessType::Read => {
                if successor.access_type.load() == AccessType::Read {
                    successor.ready();
                }
            }
            ResourceStatus::Done => successor.ready(),
            _ => {} // do nothing, the successor will be notified when anything changes
        }
    }

    pub fn ready(self: &Arc<Self>) {
        if self
            .status
            .compare_exchange(ResourceStatus::Waiting, ResourceStatus::Ready)
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
            .compare_exchange(ResourceStatus::Ready, ResourceStatus::Done)
            .is_ok()
        {
            if let Some(successor) = self.successor.load().upgrade() {
                successor.ready();
            }
        }
    }
}
