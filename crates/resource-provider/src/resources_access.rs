use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_atomic::AtomicWeak;
use kas_l2_resource::{ResourceAccess, ResourceConsumer};

use crate::ResourcesConsumer;

pub struct ResourcesAccess<C: ResourcesConsumer> {
    consumer: AtomicWeak<C>,
    resources: Vec<AtomicWeak<ResourceAccess<Self>>>,
    pending_resources: AtomicU64,
}

impl<C: ResourcesConsumer> ResourcesAccess<C> {
    pub fn new(size: usize) -> Self {
        Self {
            pending_resources: AtomicU64::new(size as u64),
            consumer: AtomicWeak::default(),
            resources: (0..size).map(|_| AtomicWeak::default()).collect(),
        }
    }

    pub fn init(self: &Arc<Self>, consumer: &Arc<C>) {
        self.consumer.store(Arc::downgrade(consumer));

        if self.pending_resources.load(Ordering::Acquire) == 0 {
            consumer.resources_available();
        }
    }

    pub fn release(&self) {
        for resource in &self.resources {
            resource.load().upgrade().unwrap().done()
        }
    }
}

impl<C: ResourcesConsumer> ResourceConsumer for ResourcesAccess<C> {
    type ConsumerGuardID = usize;
    fn notify(self: &Arc<Self>, guard: Arc<ResourceAccess<ResourcesAccess<C>>>) {
        self.resources
            .get(guard.consumer_id)
            .unwrap()
            .store(Arc::downgrade(&guard));

        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(consumer) = self.consumer.load().upgrade() {
                consumer.resources_available();
            }
        }
    }
}
