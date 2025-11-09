use std::sync::Arc;

use crossbeam_deque::Worker as WorkerQueue;
use crossbeam_queue::ArrayQueue;

mod executor;
mod worker;
mod workers_api;

pub use executor::Executor;
pub use worker::Worker;
pub use workers_api::WorkersApi;

pub trait ExecutionTask<V>: Clone + Send + Sync + 'static {
    fn execute(&self, vm: &V);
}

pub trait ExecutionBatchQueue<T>: Send
where
    T: Send + 'static,
{
    type Batch: Clone + Send + Sync + 'static;

    fn new(inbox: Arc<ArrayQueue<Self::Batch>>) -> Self;
    fn steal(&mut self, worker_queue: &WorkerQueue<T>) -> Option<T>;
}
