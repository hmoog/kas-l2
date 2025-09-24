use std::sync::Arc;

use kas_l2_core::{
    atomic::AtomicAsyncLatch,
    resources::{AtomicAccess, AtomicAccessor},
    transactions::{Transaction, TransactionProcessor},
};
use tap::Tap;

use crate::BatchAPI;

pub struct ScheduledTransaction<T: Transaction> {
    resources: Arc<AtomicAccess<T, Self>>,
    transaction: T,
    batch_api: Arc<BatchAPI<T>>,
    was_processed: AtomicAsyncLatch,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub fn new(
        resources: Arc<AtomicAccess<T, Self>>,
        transaction: T,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            resources,
            transaction,
            batch_api,
            was_processed: AtomicAsyncLatch::new(),
        })
        .tap(|this| this.resources.init_consumer(this))
    }

    pub fn process<F: TransactionProcessor<T>>(&self, processor: &F) {
        if self.was_processed.open() {
            processor(&self.transaction, &mut [] /* , &self.resources */);
            self.resources.release();
            self.batch_api.transaction_done();
        }
    }
}

impl<T: Transaction> AtomicAccessor for ScheduledTransaction<T> {
    fn available(self: &Arc<Self>) {
        self.batch_api.schedule_transaction(self.clone())
    }
}
