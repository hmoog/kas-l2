use std::sync::Arc;

use kas_l2_runtime_core::{
    atomic::AtomicOptionArc,
    resources::{Consumer, Resources},
    transactions::{Transaction, TransactionProcessor},
};
use tap::Tap;

use crate::BatchAPI;

pub struct ScheduledTransaction<T: Transaction> {
    resources: AtomicOptionArc<Resources<T, Self>>,
    transaction: T,
    batch_api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub fn new(
        resources: Arc<Resources<T, Self>>,
        transaction: T,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            resources: AtomicOptionArc::new(Some(resources.clone())),
            transaction,
            batch_api,
        })
        .tap(|this| resources.init_consumer(this))
    }

    pub fn process<F: TransactionProcessor<T>>(self: Arc<Self>, processor: &F) {
        if let Some(resources) = self.resources.take() {
            resources.consume(|h| processor(&self.transaction, h));
            self.batch_api.transaction_done();
        }
    }
}

impl<T: Transaction> Consumer for ScheduledTransaction<T> {
    fn resources_available(self: &Arc<Self>) {
        self.batch_api.schedule_transaction(self.clone())
    }
}
