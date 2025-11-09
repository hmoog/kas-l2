mod batch_queue;
mod executor;
mod worker;
mod workers_api;

pub use batch_queue::BatchQueue;
pub use executor::Executor;
pub use worker::Worker;
pub use workers_api::WorkersApi;

use crossbeam_deque::Worker as WorkerQueue;

pub trait ExecutionTask<V>: Clone + Send + 'static {
    fn execute_with(&self, vm: &V);
}

pub trait TaskBatch<T>: Clone + Send + Sync + 'static {
    fn steal_available_task(&self, worker: &WorkerQueue<T>) -> Option<T>;

    fn is_depleted(&self) -> bool;
}
