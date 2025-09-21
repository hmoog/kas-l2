use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_atomic::AtomicWeak;

use crate::{
    ResourcesConsumer,
    resource::{resource_consumer::ResourceConsumer, resource_provider::ResourceProvider},
};

pub struct ResourcesProvider<C: ResourcesConsumer> {
    consumer: AtomicWeak<C>,
    resources: Vec<AtomicWeak<ResourceProvider<Self>>>,
    pending_resources: AtomicU64,
}

impl<C: ResourcesConsumer> ResourcesProvider<C> {
    pub fn new(size: usize) -> Self {
        Self {
            pending_resources: AtomicU64::new(size as u64),
            consumer: AtomicWeak::default(),
            resources: (0..size).map(|_| AtomicWeak::default()).collect(),
        }
    }

    pub fn init_consumer(self: &Arc<Self>, consumer: &Arc<C>) {
        self.consumer.store(Arc::downgrade(consumer));

        if self.pending_resources.load(Ordering::Acquire) == 0 {
            consumer.resources_available();
        }
    }

    pub fn release(&self) {
        for resource in &self.resources {
            if let Some(resource) = resource.load().upgrade() {
                resource.done();
            }
        }
    }
}

impl<C: ResourcesConsumer> ResourceConsumer for ResourcesProvider<C> {
    type ResourceID = usize;
    fn notify(self: &Arc<Self>, resource: Arc<ResourceProvider<ResourcesProvider<C>>>) {
        self.resources
            .get(resource.consumer.1)
            .unwrap()
            .store(Arc::downgrade(&resource));

        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(consumer) = self.consumer.load().upgrade() {
                consumer.resources_available();
            }
        }
    }
}
