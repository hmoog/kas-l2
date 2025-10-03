use tap::Tap;

use crate::{AccessedResource, RuntimeTxRef, Transaction};

pub(crate) struct Resource<T: Transaction> {
    last_access: Option<AccessedResource<T>>,
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
    ) -> AccessedResource<T> {
        AccessedResource::new(access, tx_ref, self.last_access.take())
            .tap(|this| self.last_access = Some(this.clone()))
    }

    pub(crate) fn last_access(&self) -> Option<&AccessedResource<T>> {
        self.last_access.as_ref()
    }
}
