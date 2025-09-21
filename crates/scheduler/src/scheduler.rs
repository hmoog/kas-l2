use std::sync::Arc;

use crate::{Batch, ResourcesManager, Transaction};

pub struct Scheduler<T: Transaction> {
    resource_provider: ResourcesManager<T>,
}

impl<T: Transaction> Scheduler<T> {
    pub fn new(resource_provider: ResourcesManager<T>) -> Self {
        Self { resource_provider }
    }

    pub fn schedule(&mut self, tasks: Vec<T>) -> Arc<Batch<T>> {
        Arc::new(Batch::new(tasks, &mut self.resource_provider))
    }
}
