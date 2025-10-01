use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use tap::Tap;

use crate::{
    BatchAPI, Storage, Transaction, TransactionProcessor,
    resources::{
        resource_access::ResourceAccess, resource_handle::ResourceHandle,
        resource_provider::ResourceProvider,
    },
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

    pub(crate) fn new<K: Storage<T::ResourceID>>(
        batch: Arc<BatchAPI<T>>,
        resources: &mut ResourceProvider<T, K>,
        transaction: T,
    ) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<ScheduledTransaction<T>>| {
            let resources = resources.provide(&transaction, this);
            Self {
                pending_resources: AtomicU64::new(resources.len() as u64),
                batch,
                transaction,
                resources,
            }
        })
        .tap(|this| {
            for resource in this.resources() {
                resource.init(|r| resources.load_from_storage(r));
            }
        })
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
