use std::sync::Arc;

use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_core::TransactionProcessor;
use kas_l2_resource_provider::{ResourcesAccess, ResourcesConsumer};
use tap::Tap;

use crate::{BatchAPI, Transaction};

pub struct ScheduledTransaction<T: Transaction> {
    resources: Arc<ResourcesAccess<Self>>,
    transaction: T,
    batch_api: Arc<BatchAPI<T>>,
    was_processed: AtomicAsyncLatch,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub fn new(
        resources: Arc<ResourcesAccess<Self>>,
        transaction: T,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            resources,
            transaction,
            batch_api,
            was_processed: AtomicAsyncLatch::new(),
        })
        .tap(|this| this.resources.wire_up_consumer(this))
    }

    pub fn process<F: TransactionProcessor<T>>(&self, processor: &F) {
        if self.was_processed.open() {
            processor(&self.transaction /* , &self.resources */);
            self.resources.release();
            self.batch_api.transaction_done();
        }
    }
}

impl<T: Transaction> ResourcesConsumer for ScheduledTransaction<T> {
    fn resources_available(self: &Arc<Self>) {
        self.batch_api.schedule_transaction(self.clone())
    }
}
