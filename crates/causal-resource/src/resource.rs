use std::sync::{Arc, Weak};

use crate::{access_type::AccessType, consumer::Consumer, resource_guard::ResourceGuard};

pub struct Resource<N: Consumer> {
    last_guard: Option<Arc<ResourceGuard<N>>>,
}

impl<N: Consumer> Resource<N> {
    pub fn new() -> Self {
        Self { last_guard: None }
    }

    pub fn access(
        &mut self,
        notifier: Weak<N>,
        access_type: AccessType,
    ) -> (Arc<ResourceGuard<N>>, Option<Arc<ResourceGuard<N>>>) {
        let new_guard = Arc::new(ResourceGuard::new(notifier, access_type));
        (new_guard.clone(), self.last_guard.replace(new_guard))
    }

    /// Returns true if the latest guard was requested by the exact same notifier instance.
    pub fn was_last_accessed_by(&self, notifier: &Weak<N>) -> bool {
        if let Some(latest_guard) = &self.last_guard {
            return Weak::ptr_eq(&latest_guard.notifier.load(), notifier);
        }
        false
    }
}
