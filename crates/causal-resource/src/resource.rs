use std::sync::{Arc, Weak};

use crate::{access_type::AccessType, guard_consumer::GuardConsumer, guard::Guard};

pub struct Resource<C: GuardConsumer> {
    last_guard: Option<Arc<Guard<C>>>,
}

impl<C: GuardConsumer> Resource<C> {
    pub fn new() -> Self {
        Self { last_guard: None }
    }

    pub fn access(
        &mut self,
        consumer: Weak<C>,
        consumer_key: C::GuardID,
        access_type: AccessType,
    ) -> (Arc<Guard<C>>, Option<Arc<Guard<C>>>) {
        let new_guard = Arc::new(Guard::new(consumer, consumer_key, access_type));
        (new_guard.clone(), self.last_guard.replace(new_guard))
    }

    /// Returns true if the latest guard was requested by the exact same notifier instance.
    pub fn was_last_accessed_by(&self, notifier: &Weak<C>) -> bool {
        if let Some(latest_guard) = &self.last_guard {
            return Weak::ptr_eq(&latest_guard.consumer.load(), notifier);
        }
        false
    }
}
