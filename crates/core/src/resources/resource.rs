use std::sync::{Arc, Weak};

use crate::{
    resources::{AtomicAccess, AtomicAccessor, access::Access},
    transactions::Transaction,
};

pub struct Resource<T: Transaction, C: AtomicAccessor> {
    last_access: Option<Arc<Access<T, C>>>,
}

impl<T: Transaction, C: AtomicAccessor> Default for Resource<T, C> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<T: Transaction, C: AtomicAccessor> Resource<T, C> {
    pub fn access(
        &mut self,
        metadata: T::AccessMetadata,
        consumer: (Weak<AtomicAccess<T, C>>, usize),
    ) -> Arc<Access<T, C>> {
        let access = Arc::new(Access::new(metadata, consumer, self.last_access.take()));
        self.last_access = Some(access.clone());
        access
    }

    pub fn last_accessed_by(&self, atomic_access: &Weak<AtomicAccess<T, C>>) -> bool {
        let Some(last_access) = self.last_access.as_ref() else {
            return false;
        };

        Weak::ptr_eq(&last_access.atomic_ref().0, atomic_access)
    }
}
