use std::sync::{Arc, Weak};

use crate::{ResourcesConsumer, ResourcesProvider, resource_provider::ResourceProvider, Transaction};

pub struct Resource<T: Transaction, C: ResourcesConsumer> {
    last_provider: Option<Arc<ResourceProvider<T, C>>>,
}

impl<T: Transaction, C: ResourcesConsumer> Resource<T, C> {
    pub fn new() -> Self {
        Self {
            last_provider: None,
        }
    }

    pub fn provide(
        &mut self,
        access_metadata: T::AccessMetadata,
        consumer: (Weak<ResourcesProvider<T, C>>, usize),
    ) -> Arc<ResourceProvider<T, C>> {
        let guard = Arc::new(ResourceProvider::new(
            access_metadata,
            consumer,
            self.last_provider.take(),
        ));
        self.last_provider = Some(guard.clone());
        guard
    }

    pub fn was_last_accessed_by(&self, consumer: &Weak<ResourcesProvider<T, C>>) -> bool {
        self.last_provider
            .as_ref()
            .is_some_and(|p| p.has_consumer(consumer))
    }
}
