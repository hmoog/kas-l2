use std::{sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Injector, Steal, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_runtime_scheduler::{Batch, ScheduledTask, Task};

use crate::worker::Worker;

pub struct WorkersAPI<T: Task> {
    pub(crate) stealers: Vec<Stealer<Arc<ScheduledTask<T>>>>,
    pub(crate) unparkers: Vec<Unparker>,
    pub(crate) injectors: Vec<Arc<Injector<Arc<Batch<T>>>>>,
    pub(crate) shutdown: AtomicAsyncLatch,
}

impl<T: Task> WorkersAPI<T> {
    pub fn new_with_workers(worker_count: usize) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        let mut this = Self {
            stealers: vec![],
            unparkers: vec![],
            injectors: vec![],
            shutdown: AtomicAsyncLatch::new(),
        };

        // create workers
        let workers: Vec<Worker<T>> = (0..worker_count)
            .map(|id| {
                let worker = Worker::new(id);
                this.stealers.push(worker.local_queue.stealer());
                this.unparkers.push(worker.parker.unparker().clone());
                this.injectors.push(worker.injector.clone());
                worker
            })
            .collect();

        // start workers (they will immediately park)
        let this = Arc::new(this);
        let handles = workers.into_iter().map(|w| w.start(this.clone())).collect();

        (this, handles)
    }

    pub fn inject_batch(self: &Arc<Self>, batch: Arc<Batch<T>>) {
        for (injector, unparker) in self.injectors.iter().zip(&self.unparkers) {
            injector.push(batch.clone()); // broadcast Arc to each worker
            unparker.unpark(); // wake worker so it discovers sooner
        }
    }

    pub fn steal(self: &Arc<Self>, worker_id: usize) -> Option<Arc<ScheduledTask<T>>> {
        for (id, other) in self.stealers.iter().enumerate() {
            if id != worker_id {
                loop {
                    match other.steal() {
                        Steal::Success(task) => return Some(task),
                        Steal::Retry => continue,
                        Steal::Empty => break,
                    }
                }
            }
        }
        None
    }
}
