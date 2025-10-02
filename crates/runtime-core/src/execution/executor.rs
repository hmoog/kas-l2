use std::thread::JoinHandle;

use crate::{BatchApi, Transaction, TransactionProcessor, execution::workers_api::WorkersApi};

pub struct Executor<T: Transaction> {
    workers: WorkersApi<T>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: Transaction> Executor<T> {
    pub fn new<P: TransactionProcessor<T>>(worker_count: usize, processor: P) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, processor);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: BatchApi<T>) {
        self.workers.inject_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();

        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
