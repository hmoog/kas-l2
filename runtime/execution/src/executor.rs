use std::thread::JoinHandle;

use crate::{ExecutionTask, TaskBatch, WorkersApi};

pub struct Executor<T, B, V>
where
    T: ExecutionTask<V> + Clone + Send + 'static,
    B: TaskBatch<T>,
    V: Clone + Send + Sync + 'static,
{
    workers: WorkersApi<T, B, V>,
    handles: Vec<JoinHandle<()>>,
}

impl<T, B, V> Executor<T, B, V>
where
    T: ExecutionTask<V> + Clone + Send + 'static,
    B: TaskBatch<T>,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(worker_count: usize, vm: V) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, vm);
        Self { workers, handles }
    }

    pub fn execute(&self, batch: B) {
        self.workers.push_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
