use std::sync::{Arc, Weak};

use kas_l2_core::{AccessType, Transaction};

use crate::resource::{resource_consumer::ResourceConsumer, resource_provider::ResourceProvider};

pub struct Resource<T: Transaction, C: ResourceConsumer<T>> {
    last_provider: Option<Arc<ResourceProvider<T, C>>>,
}

impl<T: Transaction, C: ResourceConsumer<T>> Resource<T, C> {
    pub fn new() -> Self {
        Self {
            last_provider: None,
        }
    }

    pub fn provide(
        &mut self,
        (consumer, id): (Weak<C>, C::ResourceID),
        access_type: AccessType,
    ) -> Arc<ResourceProvider<T, C>> {
        let guard = Arc::new(ResourceProvider::new(
            self.last_provider.take(),
            consumer,
            id,
            access_type,
        ));
        self.last_provider = Some(guard.clone());
        guard
    }

    pub fn was_last_accessed_by(&self, consumer: &Weak<C>) -> bool {
        self.last_provider
            .as_ref()
            .is_some_and(|guard| Weak::ptr_eq(&guard.consumer.0.load(), consumer))
    }
}

pub mod access_status;
pub mod resource_consumer;
pub mod resource_provider;
