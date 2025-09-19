use std::sync::Arc;

use kas_l2_resource_provider::ResourceProvider;

use crate::{BatchAPI, scheduled_task::ScheduledTask, task::Task};

pub struct Batch<T: Task> {
    scheduled_tasks: Vec<Arc<ScheduledTask<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<T: Task> Batch<T> {
    pub fn new(
        tasks: Vec<T>,
        resources: &mut ResourceProvider<T::ResourceID, ScheduledTask<T>>,
    ) -> Self {
        let mut this = Self {
            scheduled_tasks: Vec::new(),
            api: Arc::new(BatchAPI::new(tasks.len() as u64)),
        };
        for element in tasks {
            this.scheduled_tasks
                .push(ScheduledTask::new(element, resources, this.api.clone()))
        }
        this
    }

    pub fn api(&self) -> Arc<BatchAPI<T>> {
        self.api.clone()
    }
}
