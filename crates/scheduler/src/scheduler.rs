use std::sync::Arc;

use kas_l2_core::transactions::Transaction;

use crate::{Batch, BatchAPI, ResourcesManager};

pub struct Scheduler<T: Transaction> {
    resource_provider: ResourcesManager<T>,
    last_batch: Option<Arc<Batch<T>>>,
}

impl<T: Transaction> Scheduler<T> {
    pub fn new(resource_provider: ResourcesManager<T>) -> Self {
        Self {
            resource_provider,
            last_batch: None,
        }
    }

    pub fn schedule(&mut self, tasks: Vec<T>) -> Arc<BatchAPI<T>> {
        let batch = Arc::new(Batch::new(
            self.last_batch.take(),
            tasks,
            &mut self.resource_provider,
        ));
        let api = batch.api();
        self.last_batch = Some(batch);
        api
    }
}
