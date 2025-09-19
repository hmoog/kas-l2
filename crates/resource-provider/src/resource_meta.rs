use std::sync::{Arc, Weak};

use kas_l2_resource::{AccessType, ResourceAccess, ResourceConsumer};

pub struct ResourceMeta<C: ResourceConsumer> {
    last_guard: Option<Arc<ResourceAccess<C>>>,
}

impl<C: ResourceConsumer> ResourceMeta<C> {
    pub fn new() -> Self {
        Self { last_guard: None }
    }

    pub fn access(
        &mut self,
        consumer: Weak<C>,
        id: C::ResourceID,
        acc: AccessType,
    ) -> Arc<ResourceAccess<C>> {
        let guard = Arc::new(ResourceAccess::new(
            self.last_guard.take(),
            consumer,
            id,
            acc,
        ));
        self.last_guard = Some(guard.clone());
        guard
    }

    pub fn was_accessed_by(&self, consumer: &Weak<C>) -> bool {
        self.last_guard
            .as_ref()
            .is_some_and(|guard| Weak::ptr_eq(&guard.consumer.load(), consumer))
    }
}
