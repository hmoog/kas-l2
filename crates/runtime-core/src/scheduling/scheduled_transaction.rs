use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    BatchAPI, Transaction, TransactionProcessor,
    resources::{resource_access::ResourceAccess, resource_handle::ResourceHandle},
};

pub struct ScheduledTransaction<T: Transaction> {
    batch: Arc<BatchAPI<T>>,
    resources: Vec<Arc<ResourceAccess<T>>>,
    pending_resources: AtomicU64,
    transaction: T,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub fn resources(&self) -> &[Arc<ResourceAccess<T>>] {
        &self.resources
    }

    pub(crate) fn new(
        batch: Arc<BatchAPI<T>>,
        resources: Vec<Arc<ResourceAccess<T>>>,
        transaction: T,
    ) -> Self {
        Self {
            pending_resources: AtomicU64::new(resources.len() as u64),
            batch,
            transaction,
            resources,
        }
    }

    pub(crate) fn process<F: TransactionProcessor<T>>(self: Arc<Self>, processor: &F) {
        processor(&self.transaction, &mut self.resource_handles());
        self.batch.decrease_pending_transactions();
    }

    pub(crate) fn decrease_pending_resources(self: Arc<Self>) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.batch.push_available(&self)
        }
    }

    fn resource_handles(&self) -> Vec<ResourceHandle<'_, T>> {
        self.resources.iter().map(ResourceHandle::new).collect()
    }
}
