use std::sync::Arc;

use tap::Tap;

use crate::{RuntimeTxRef, Transaction, resources::accessed_resource::AccessedResource};

pub(crate) struct Resource<T: Transaction> {
    last_access: Option<Arc<AccessedResource<T>>>,
}

impl<T: Transaction> Default for Resource<T> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<T: Transaction> Resource<T> {
    pub(crate) fn access(
        &mut self,
        access: T::Access,
        tx_ref: RuntimeTxRef<T>,
    ) -> Arc<AccessedResource<T>> {
        AccessedResource::new(access, tx_ref, self.last_access.take())
            .tap(|this| self.last_access = Some(this.clone()))
    }

    pub(crate) fn was_accessed_by(&self, tx_ref: &RuntimeTxRef<T>) -> bool {
        match self.last_access.as_ref() {
            Some(last_resource) => last_resource.parent_eq(tx_ref),
            None => false,
        }
    }
}
