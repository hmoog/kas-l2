use std::sync::Arc;

use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_resource_provider::{ResourcesAccess, ResourcesConsumer};

use crate::{BatchAPI, ResourceProvider, Transaction};

pub struct ScheduledTask<T: Transaction> {
    task: T,
    resources: Arc<ResourcesAccess<ScheduledTask<T>>>,
    batch_api: Arc<BatchAPI<T>>,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> ScheduledTask<T> {
    pub(crate) fn new(
        task: T,
        resources: &mut ResourceProvider<T>,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Arc<Self> {
        let this = Arc::new(Self {
            resources: resources.resources(task.write_locks(), task.read_locks()),
            task,
            batch_api,
            is_done: AtomicAsyncLatch::new(),
        });
        this.resources.init(&this);
        this
    }

    pub fn task(&self) -> &T {
        &self.task
    }

    pub fn mark_done(&self) {
        if self.is_done.open() {
            self.resources.release();
            self.batch_api.decrease_pending_tasks();
        }
    }
}

impl<T: Transaction> ResourcesConsumer for ScheduledTask<T> {
    fn resources_available(self: &Arc<Self>) {
        self.batch_api.scheduled_tasks.push(self.clone())
    }
}
