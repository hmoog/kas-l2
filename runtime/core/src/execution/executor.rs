use std::thread::JoinHandle;

use kas_l2_storage_manager::Store;

use crate::{Batch, RuntimeState, WorkersApi, vm::VM};

pub struct Executor<S: Store<StateSpace = RuntimeState>, V: VM> {
    workers: WorkersApi<S, V>,
    handles: Vec<JoinHandle<()>>,
}

impl<S: Store<StateSpace = RuntimeState>, V: VM> Executor<S, V> {
    pub fn new(worker_count: usize, vm: V) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, vm);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: Batch<S, V>) {
        self.workers.push_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
