use std::thread::JoinHandle;

use kas_l2_storage_manager::Store;

use crate::{Batch, RuntimeState, Vm, WorkersApi};

pub struct Executor<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    workers: WorkersApi<S, VM>,
    handles: Vec<JoinHandle<()>>,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> Executor<S, VM> {
    pub fn new(worker_count: usize, vm: VM) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, vm);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: Batch<S, VM>) {
        self.workers.push_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
