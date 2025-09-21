use std::sync::{Arc, Weak};

use crate::resource::{
    access_type::AccessType, resource_consumer::ResourceConsumer,
    resource_provider::ResourceProvider,
};

pub struct Resource<C: ResourceConsumer> {
    last_provider: Option<Arc<ResourceProvider<C>>>,
}

impl<C: ResourceConsumer> Resource<C> {
    pub fn new() -> Self {
        Self {
            last_provider: None,
        }
    }

    pub fn provide(
        &mut self,
        (consumer, id): (Weak<C>, C::ResourceID),
        access_type: AccessType,
    ) -> Arc<ResourceProvider<C>> {
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
pub mod access_type;
pub mod resource_consumer;
pub mod resource_provider;
