use std::sync::{Arc, Weak};

use tap::Tap;

use crate::{
    Transaction, resources::resource_access::ResourceAccess,
    scheduling::scheduled_transaction::ScheduledTransaction,
};

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
        parent: Weak<ScheduledTransaction<T>>,
    ) -> Arc<ResourceAccess<T>> {
        ResourceAccess::new(access_metadata, parent, self.last_access.take())
            .tap(|this| self.last_access = Some(this.clone()))
    }

    pub(crate) fn was_accessed_by(&self, transaction: &Weak<ScheduledTransaction<T>>) -> bool {
        match self.last_access.as_ref() {
            Some(last_resource) => last_resource.parent_eq(transaction),
            None => false,
        }
    }
}
