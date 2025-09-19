use std::sync::Arc;

use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_resource_provider::{ResourcesAccess, ResourcesConsumer};

use crate::{BatchAPI, Transaction};

pub struct ScheduledTransaction<T: Transaction> {
    transaction: T,
    resources: Arc<ResourcesAccess<ScheduledTransaction<T>>>,
    batch_api: Arc<BatchAPI<T>>,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> ScheduledTransaction<T> {
    pub(crate) fn new(
        transaction: T,
        resources: Arc<ResourcesAccess<ScheduledTransaction<T>>>,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Arc<Self> {
        let this = Arc::new(Self {
            resources,
            transaction,
            batch_api,
            is_done: AtomicAsyncLatch::new(),
        });
        this.resources.init(&this);
        this
    }

    pub fn transaction(&self) -> &T {
        &self.transaction
    }

    pub fn mark_done(&self) {
        if self.is_done.open() {
            self.resources.release();
            self.batch_api.decrease_pending_tasks();
        }
    }
}

impl<T: Transaction> ResourcesConsumer for ScheduledTransaction<T> {
    fn resources_available(self: &Arc<Self>) {
        self.batch_api.scheduled_tasks.push(self.clone())
    }
}
