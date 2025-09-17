use std::sync::Arc;

use kas_l2_causal_resource::Provider;

use crate::{ScheduledTask, batch::Batch, task::Task};

pub struct Scheduler<T: Task> {
    resource_provider: Provider<T::ResourceID, ScheduledTask<T>>,
}

impl<T: Task> Scheduler<T> {
    pub fn new() -> Self {
        Self {
            resource_provider: Provider::new(),
        }
    }

    pub fn schedule(&mut self, elements: Vec<T>) -> Arc<Batch<T>> {
        Arc::new(Batch::new(elements, &mut self.resource_provider))
    }
}

impl<T: Task> Default for Scheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}
