use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_resource_provider::{ResourceProvider, ResourcesAccess, ResourcesConsumer};

use crate::{BatchAPI, task::Task};

pub struct ScheduledTask<T: Task> {
    resources: Arc<ResourcesAccess<ScheduledTask<T>>>,
    task: T,
    batch_api: Arc<BatchAPI<T>>,
    is_done: AtomicBool,
}

impl<T: Task> ScheduledTask<T> {
    pub(crate) fn new(
        task: T,
        resources: &mut ResourceProvider<T::ResourceID, ScheduledTask<T>>,
        batch_api: Arc<BatchAPI<T>>,
    ) -> Arc<Self> {
        let this = Arc::new(Self {
            resources: resources.resources(task.write_locks(), task.read_locks()),
            task,
            batch_api,
            is_done: AtomicBool::new(false),
        });
        this.resources.init(&this);
        this
    }

    pub fn task(&self) -> &T {
        &self.task
    }

    pub fn mark_done(&self) {
        if self
            .is_done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.resources.release();

            if self.batch_api.count.fetch_sub(1, Ordering::AcqRel) == 1 {
                self.batch_api.done.open()
            }
        }
    }
}

impl<T: Task> ResourcesConsumer for ScheduledTask<T> {
    fn resources_available(self: &Arc<Self>) {
        self.batch_api.ready.push(self.clone())
    }
}
