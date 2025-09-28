use std::sync::{Arc, Weak};

use crate::{ScheduledTransaction, resource::Resource, transaction::Transaction};

pub(crate) struct ResourceManager<T: Transaction> {
    last_resource: Option<Arc<Resource<T>>>,
}

impl<T: Transaction> Default for ResourceManager<T> {
    fn default() -> Self {
        Self {
            last_resource: None,
        }
    }
}

impl<T: Transaction> ResourceManager<T> {
    pub(crate) fn provide_resource(
        &mut self,
        metadata: T::AccessMetadata,
        scheduled_transaction: Weak<ScheduledTransaction<T>>,
    ) -> Arc<Resource<T>> {
        let access = Resource::new(scheduled_transaction, self.last_resource.take(), metadata);
        self.last_resource = Some(access.clone());
        access
    }

    pub(crate) fn has_duplicate_access(&self, resources: &Weak<ScheduledTransaction<T>>) -> bool {
        match self.last_resource.as_ref() {
            Some(last_resource) => last_resource.belongs_to(resources),
            None => false,
        }
    }
}
