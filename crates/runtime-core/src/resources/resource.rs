use std::sync::Arc;

use tap::Tap;

use crate::{ScheduledTransactionRef, Transaction, resources::resource_access::ResourceAccess};

pub(crate) struct Resource<T: Transaction> {
    last_access: Option<Arc<ResourceAccess<T>>>,
}

impl<T: Transaction> Default for Resource<T> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<T: Transaction> Resource<T> {
    pub(crate) fn access(
        &mut self,
        access_metadata: T::AccessMetadata,
        parent: ScheduledTransactionRef<T>,
    ) -> Arc<ResourceAccess<T>> {
        ResourceAccess::new(access_metadata, parent, self.last_access.take())
            .tap(|this| self.last_access = Some(this.clone()))
    }

    pub(crate) fn was_accessed_by(&self, transaction: &ScheduledTransactionRef<T>) -> bool {
        match self.last_access.as_ref() {
            Some(last_resource) => last_resource.parent_eq(transaction),
            None => false,
        }
    }
}
