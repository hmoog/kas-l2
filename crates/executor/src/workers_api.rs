use std::{sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Injector, Steal, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_runtime_core::{
    atomic::AtomicAsyncLatch,
    transactions::{Transaction, TransactionProcessor},
};
use kas_l2_scheduler::{BatchAPI, ScheduledTransaction};

use crate::worker::Worker;

pub struct WorkersAPI<T: Transaction> {
    stealers: Vec<Stealer<Arc<ScheduledTransaction<T>>>>,
    unparkers: Vec<Unparker>,
    injectors: Vec<Arc<Injector<Arc<BatchAPI<T>>>>>,
    shutdown: AtomicAsyncLatch,
}

impl<T: Transaction> WorkersAPI<T> {
    pub fn new_with_workers<P: TransactionProcessor<T>>(
        worker_count: usize,
        processor: P,
    ) -> (Arc<Self>, Vec<JoinHandle<()>>) {
        // create owned instance
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

        // return shared instance + handles
        (this, handles)
    }

    pub fn inject_batch(self: &Arc<Self>, batch: Arc<BatchAPI<T>>) {
        for (injector, unparker) in self.injectors.iter().zip(&self.unparkers) {
            injector.push(batch.clone());
            unparker.unpark();
        }
    }

    pub fn steal_task_from_other_workers(
        self: &Arc<Self>,
        worker_id: usize,
    ) -> Option<Arc<ScheduledTransaction<T>>> {
        // TODO: randomize stealer selection
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
        // trigger shutdown signal
        self.shutdown.open();

        // wake all workers so they can exit
        for unparker in &self.unparkers {
            unparker.unpark();
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.is_open()
    }
}
