use std::sync::Arc;

use crate::{Batch, ResourceProvider, Task};

pub struct Scheduler<T: Task> {
    resource_provider: ResourceProvider<T>,
}

impl<T: Task> Scheduler<T> {
    pub fn new() -> Self {
        Self {
            resource_provider: ResourceProvider::new(),
        }
    }

    pub fn schedule(&mut self, tasks: Vec<T>) -> Arc<Batch<T>> {
        Arc::new(Batch::new(tasks, &mut self.resource_provider))
    }
}

impl<T: Task> Default for Scheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}
