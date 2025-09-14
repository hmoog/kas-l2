use std::{sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Injector, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_runtime_scheduler::{Batch, ScheduledTask, Task};

use crate::worker::Worker;

/// Manages a pool of workers and facilitates task distribution among them.
pub struct WorkerManager<T: Task> {
    pub(crate) stealers: Vec<Stealer<Arc<ScheduledTask<T>>>>,
    pub(crate) unparkers: Vec<Unparker>,
    pub(crate) batch_injectors: Vec<Arc<Injector<Arc<Batch<T>>>>>,
    pub(crate) shutdown: AtomicAsyncLatch,
}

/// Public API
impl<T: Task> WorkerManager<T> {
    /// Spawns a specified number of worker threads and returns the manager along with their join
    /// handles.
    pub fn spawn(worker_count: usize) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        let mut this = Self {
            stealers: vec![],
            unparkers: vec![],
            batch_injectors: vec![],
            shutdown: AtomicAsyncLatch::new(),
        };

        let workers = this.create_workers(worker_count);
        this.start_workers(workers)
    }

    /// Injects a batch of tasks into all workers for processing.
    pub fn inject_batch(&self, batch: Arc<Batch<T>>) {
        for (batch_injector, unparker) in self.batch_injectors.iter().zip(&self.unparkers) {
            batch_injector.push(batch.clone()); // broadcast Arc to each worker
            unparker.unpark(); // wake worker so it discovers sooner
        }
    }
}

/// Internal methods
impl<T: Task> WorkerManager<T> {
    /// Creates the specified number of workers and initializes their associated components.
    fn create_workers(&mut self, worker_count: usize) -> Vec<Worker<T>> {
        (0..worker_count)
            .map(|id| {
                let worker = Worker::new(id);
                self.stealers.push(worker.queue.stealer());
                self.unparkers.push(worker.parker.unparker().clone());
                self.batch_injectors.push(worker.batch_injector.clone());
                worker
            })
            .collect()
    }

    /// Starts the provided workers, returning an `Arc` to the manager and their join handles.
    fn start_workers(self, workers: Vec<Worker<T>>) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        let this = Arc::new(self);
        let handles = workers.into_iter().map(|w| w.start(this.clone())).collect();
        (this, handles)
    }
}
