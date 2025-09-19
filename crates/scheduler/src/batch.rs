use std::sync::Arc;

use kas_l2_resource::{ResourceProvider};

use crate::{BatchAPI, scheduled_task::ScheduledTask, task::Task};

pub struct Batch<T: Task> {
    scheduled_tasks: Vec<Arc<ScheduledTask<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<E: Task> Batch<E> {
    pub fn new(elements: Vec<E>, provider: &mut ResourceProvider<E::ResourceID, ScheduledTask<E>>) -> Self {
        let mut this = Self {
            scheduled_tasks: Vec::new(),
            api: Arc::new(BatchAPI::new(elements.len() as u64)),
        };
        for element in elements {
            this.scheduled_tasks.push(ScheduledTask::new(element, provider, this.api.clone()))
        }
        this
    }

    pub fn api(&self) -> Arc<BatchAPI<E>> {
        self.api.clone()
    }
}
