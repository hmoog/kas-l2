use std::{sync::Arc, thread::JoinHandle};

use crate::{BatchAPI, Transaction, TransactionProcessor, execution::workers_api::WorkersAPI};

pub struct Executor<T: Transaction> {
    workers: Arc<WorkersAPI<T>>,
    handles: Vec<JoinHandle<()>>,
}

impl<T: Transaction> Executor<T> {
    pub fn new<P: TransactionProcessor<T>>(worker_count: usize, processor: P) -> Self {
        let (workers, handles) = WorkersAPI::new_with_workers(worker_count, processor);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: Arc<BatchAPI<T>>) {
        self.workers.inject_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();

        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
