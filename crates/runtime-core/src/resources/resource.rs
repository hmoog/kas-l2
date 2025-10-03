use tap::Tap;

use crate::{ResourceAccess, RuntimeTxRef, Transaction};

pub(crate) struct Resource<T: Transaction> {
    last_access: Option<ResourceAccess<T>>,
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
        tx_ref: RuntimeTxRef<T>,
    ) -> ResourceAccess<T> {
        ResourceAccess::new(access_metadata, tx_ref, self.last_access.take())
            .tap(|this| self.last_access = Some(this.clone()))
    }

    pub(crate) fn last_access(&self) -> Option<&ResourceAccess<T>> {
        self.last_access.as_ref()
    }
}
