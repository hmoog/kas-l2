use std::thread::JoinHandle;

use kas_l2_storage::Store;

use crate::{Batch, RuntimeState, Transaction, TransactionProcessor, WorkersApi};

pub struct Executor<S: Store<StateSpace = RuntimeState>, Tx: Transaction> {
    workers: WorkersApi<S, Tx>,
    handles: Vec<JoinHandle<()>>,
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> Executor<S, Tx> {
    pub fn new<P: TransactionProcessor<S, Tx>>(worker_count: usize, processor: P) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, processor);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: Batch<S, Tx>) {
        self.workers.push_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
