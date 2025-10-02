use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use tap::Tap;

use crate::{
    BatchApi, Storage, Transaction, TransactionProcessor,
    resources::{
        resource_access::ResourceAccess, resource_handle::ResourceHandle,
        resource_provider::ResourceProvider,
    },
};

pub struct ScheduledTransaction<T: Transaction>(Arc<ScheduledTransactionData<T>>);

struct ScheduledTransactionData<T: Transaction> {
    batch: BatchApi<T>,
    resources: Vec<Arc<ResourceAccess<T>>>,
    pending_resources: AtomicU64,
    transaction: T,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub fn downgrade(&self) -> ScheduledTransactionRef<T> {
        ScheduledTransactionRef(Arc::downgrade(&self.0))
    }

    pub(crate) fn new<K: Storage<T::ResourceID>>(
        batch: BatchApi<T>,
        resources: &mut ResourceProvider<T, K>,
        transaction: T,
    ) -> Self {
        Self(Arc::new_cyclic(
            |this: &Weak<ScheduledTransactionData<T>>| {
                let resources =
                    resources.provide(&transaction, ScheduledTransactionRef(this.clone()));
                ScheduledTransactionData {
                    pending_resources: AtomicU64::new(resources.len() as u64),
                    batch,
                    transaction,
                    resources,
                }
            },
        ))
        .tap(|this| {
            for resource in this.resources() {
                resource.init(|r| resources.load_from_storage(r));
            }
        })
    }

    pub fn resources(&self) -> &[Arc<ResourceAccess<T>>] {
        &self.0.resources
    }

    pub(crate) fn process<F: TransactionProcessor<T>>(&self, processor: &F) {
        processor(&self.0.transaction, &mut self.resource_handles());
        self.0.batch.decrease_pending_transactions();
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.0.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.batch.push_available(&self)
        }
    }

    fn resource_handles(&self) -> Vec<ResourceHandle<'_, T>> {
        self.0.resources.iter().map(ResourceHandle::new).collect()
    }
}

impl<T: Transaction> Clone for ScheduledTransaction<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct ScheduledTransactionRef<T: Transaction>(Weak<ScheduledTransactionData<T>>);

impl<T: Transaction> ScheduledTransactionRef<T> {
    pub fn upgrade(&self) -> Option<ScheduledTransaction<T>> {
        self.0.upgrade().map(ScheduledTransaction)
    }
}

impl<T: Transaction> PartialEq for ScheduledTransactionRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Transaction> Clone for ScheduledTransactionRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
