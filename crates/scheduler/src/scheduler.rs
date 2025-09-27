use std::sync::Arc;

use kas_l2_runtime_core::{storage::KvStore, transactions::Transaction};

use crate::{Batch, BatchAPI, ResourceProvider};

pub struct Scheduler<T: Transaction, K: KvStore<T::ResourceID>> {
    resource_provider: ResourceProvider<T, K>,
    last_batch: Option<Arc<Batch<T>>>,
}

impl<T: Transaction, K: KvStore<T::ResourceID>> Scheduler<T, K> {
    pub fn new(resource_provider: ResourceProvider<T, K>) -> Self {
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
