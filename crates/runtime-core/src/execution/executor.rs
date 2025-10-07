use std::thread::JoinHandle;

use crate::{Batch, Transaction, TransactionProcessor, WorkersApi};

pub struct Executor<Tx: Transaction> {
    workers: WorkersApi<Tx>,
    handles: Vec<JoinHandle<()>>,
}

impl<Tx: Transaction> Executor<Tx> {
    pub fn new<P: TransactionProcessor<Tx>>(worker_count: usize, processor: P) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, processor);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: Batch<Tx>) {
        self.workers.push_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
