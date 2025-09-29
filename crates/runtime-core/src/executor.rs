use std::{sync::Arc, thread::JoinHandle};

use crate::{BatchAPI, Transaction, TransactionProcessor, workers_api::WorkersAPI};

pub struct Executor<T: Transaction> {
    workers_api: Arc<WorkersAPI<T>>,
    worker_handles: Vec<JoinHandle<()>>,
}

impl<T: Transaction> Executor<T> {
    pub fn new<P: TransactionProcessor<T>>(worker_count: usize, processor: P) -> Self {
        let (workers_api, worker_handles) = WorkersAPI::new_with_workers(worker_count, processor);
        Self {
            workers_api,
            worker_handles,
        }
    }

    pub fn execute(&self, batch: Arc<BatchAPI<T>>) {
        self.workers_api.inject_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers_api.shutdown();

        for handle in self.worker_handles {
            let _ = handle.join();
        }
    }
}
