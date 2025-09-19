use std::sync::Arc;

use crate::{BatchAPI, ResourceProvider, ScheduledTask, Transaction};

pub struct Batch<T: Transaction> {
    scheduled_tasks: Vec<Arc<ScheduledTask<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> Batch<T> {
    pub fn new(tasks: Vec<T>, resources: &mut ResourceProvider<T>) -> Self {
        let api = Arc::new(BatchAPI::new(tasks.len() as u64));
        Self {
            scheduled_tasks: tasks
                .into_iter()
                .map(|t| ScheduledTask::new(t, resources, api.clone()))
                .collect(),
            api,
        }
    }

    pub fn size(&self) -> usize {
        self.scheduled_tasks.len()
    }

    pub fn api(&self) -> Arc<BatchAPI<T>> {
        self.api.clone()
    }
}
