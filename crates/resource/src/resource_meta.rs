use std::sync::{Arc, Weak};

use crate::{access_type::AccessType, resource_access::ResourceAccess, resource_consumer::GuardConsumer};

pub struct ResourceMeta<C: GuardConsumer> {
    last_guard: Option<Arc<ResourceAccess<C>>>,
}

impl<C: GuardConsumer> ResourceMeta<C> {
    pub fn new() -> Self {
        Self { last_guard: None }
    }

    pub fn access(&mut self, consumer: Weak<C>, id: C::ConsumerGuardID, acc: AccessType) -> Arc<ResourceAccess<C>> {
        let guard = Arc::new(ResourceAccess::new(self.last_guard.take(), consumer, id, acc));
        self.last_guard = Some(guard.clone());
        guard
    }

    pub fn was_accessed_by(&self, consumer: &Weak<C>) -> bool {
        self.last_guard
            .as_ref()
            .is_some_and(|guard| Weak::ptr_eq(&guard.consumer.load(), consumer))
    }
}
