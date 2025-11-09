use std::{marker::PhantomData, thread::JoinHandle};

use crate::{ExecutionBatchQueue, ExecutionTask, WorkersApi};

pub struct Executor<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    workers: WorkersApi<V, T, Q>,
    handles: Vec<JoinHandle<()>>,
    marker: PhantomData<V>,
}

impl<V, T, Q> Executor<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    pub fn new(worker_count: usize, vm: V) -> Self {
        let (workers, handles) = WorkersApi::new_with_workers(worker_count, vm);
        Self { workers, handles, marker: PhantomData }
    }

    pub fn execute(&self, batch: Q::Batch) {
        self.workers.push_batch(batch);
    }

    pub fn shutdown(self) {
        self.workers.shutdown();
        for handle in self.handles {
            handle.join().expect("executor worker panicked");
        }
    }
}
