use std::{sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Injector, Steal, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_causal_scheduler::{Batch, ScheduledTask, Task};

use crate::{processor::Processor, worker::Worker};

pub struct WorkersAPI<T: Task> {
    stealers: Vec<Stealer<Arc<ScheduledTask<T>>>>,
    unparkers: Vec<Unparker>,
    injectors: Vec<Arc<Injector<Arc<Batch<T>>>>>,
    shutdown: AtomicAsyncLatch,
}

impl<T: Task> WorkersAPI<T> {
    pub fn new_with_workers<P: Processor<T>>(
        worker_count: usize,
        processor: P,
    ) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        let mut this = Self {
            stealers: vec![],
            unparkers: vec![],
            injectors: vec![],
            shutdown: AtomicAsyncLatch::new(),
        };

        // create workers
        let workers: Vec<Worker<T, P>> = (0..worker_count)
            .map(|id| {
                let worker = Worker::new(id, processor.clone());
                this.stealers.push(worker.stealer());
                this.unparkers.push(worker.unparker());
                this.injectors.push(worker.injector());
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
            injector.push(batch.clone());
            unparker.unpark();
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

    pub fn shutdown(&self) {
        self.shutdown.open();
        for unparker in &self.unparkers {
            unparker.unpark(); // wake all workers so they can exit
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.is_open()
    }
}
