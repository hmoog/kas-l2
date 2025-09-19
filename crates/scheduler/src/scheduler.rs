use std::sync::Arc;

use crate::{Batch, ResourceProvider, Transaction};

pub struct Scheduler<T: Transaction> {
    resource_provider: ResourceProvider<T>,
}

impl<T: Transaction> Scheduler<T> {
    pub fn new() -> Self {
        Self {
            resource_provider: ResourceProvider::new(),
        }
    }

    pub fn schedule(&mut self, tasks: Vec<T>) -> Arc<Batch<T>> {
        Arc::new(Batch::new(tasks, &mut self.resource_provider))
    }
}

impl<T: Transaction> Default for Scheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}
