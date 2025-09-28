use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_atomic::AtomicOptionArc;

use crate::{
    AccessHandle, AccessMetadata, AccessType, BatchAPI, Transaction, TransactionProcessor,
    resource::Resource,
};

pub struct ScheduledTransaction<T: Transaction> {
    pub(crate) resources: Vec<AtomicOptionArc<Resource<T>>>,
    pending_resources: AtomicU64,
    transaction: T,
    batch_api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub(crate) fn new(
        resources: Vec<Arc<Resource<T>>>,
        transaction: T,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Self {
        Self {
            pending_resources: AtomicU64::new(resources.len() as u64),
            resources: resources
                .into_iter()
                .map(|r| AtomicOptionArc::new(Some(r)))
                .collect(),
            transaction,
            batch_api,
        }
    }

    pub(crate) fn resources(&self) -> Vec<Arc<Resource<T>>> {
        self.resources
            .iter()
            .filter_map(AtomicOptionArc::load)
            .collect()
    }

    pub fn process<F: TransactionProcessor<T>>(self: Arc<Self>, processor: &F) {
        let resources: Vec<_> = self
            .resources
            .iter()
            .filter_map(AtomicOptionArc::take)
            .collect();
        assert_eq!(resources.len(), self.resources.len(), "missing resources");

        let mut handles: Vec<_> = resources
            .iter()
            .map(|access| AccessHandle::new(access.read_state(), access))
            .collect();

        processor(&self.transaction, &mut handles);

        for (handle, access) in handles.into_iter().zip(resources.iter()) {
            if handle.access_type() == AccessType::Write {
                access.set_written_state(handle.commit());
            }
        }

        self.batch_api.transaction_done();
    }

    pub(crate) fn decrease_pending_resources(self: Arc<Self>) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.batch_api.schedule_transaction(self.clone())
        }
    }
}
