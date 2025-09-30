use std::sync::{Arc, Weak};

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
        scheduled_transaction: Weak<ScheduledTransaction<T>>,
    ) -> Arc<ResourceAccess<T>> {
        let access = ResourceAccess::new(
            scheduled_transaction,
            self.last_access.take(),
            access_metadata,
        );
        self.last_access = Some(access.clone());
        access
    }

    pub(crate) fn was_accessed_by(&self, resources: &Weak<ScheduledTransaction<T>>) -> bool {
        match self.last_access.as_ref() {
            Some(last_resource) => last_resource.belongs_to(resources),
            None => false,
        }
    }
}
