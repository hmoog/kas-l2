use std::{sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Injector, Steal, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::AtomicAsyncLatch;

use crate::{
    BatchApi, ScheduledTransaction, Transaction, TransactionProcessor, execution::worker::Worker,
};

pub struct WorkersApi<T: Transaction>(Arc<WorkersApiData<T>>);

struct WorkersApiData<T: Transaction> {
    stealers: Vec<Stealer<ScheduledTransaction<T>>>,
    unparkers: Vec<Unparker>,
    injectors: Vec<Arc<Injector<BatchApi<T>>>>,
    shutdown: AtomicAsyncLatch,
}

impl<T: Transaction> WorkersApi<T> {
    pub fn new_with_workers<P: TransactionProcessor<T>>(
        worker_count: usize,
        processor: P,
    ) -> (Self, Vec<JoinHandle<()>>) {
        // create owned instance
        let mut data = WorkersApiData {
            stealers: vec![],
            unparkers: vec![],
            injectors: vec![],
            shutdown: AtomicAsyncLatch::new(),
        };

        // create workers
        let workers: Vec<Worker<T, P>> = (0..worker_count)
            .map(|id| {
                let worker = Worker::new(id, processor.clone());
                data.stealers.push(worker.stealer());
                data.unparkers.push(worker.unparker());
                data.injectors.push(worker.injector());
                worker
            })
            .collect();

        // start workers (they will immediately park)
        let this = Self(Arc::new(data));
        let handles = workers.into_iter().map(|w| w.start(this.clone())).collect();

        // return shared instance + handles
        (this, handles)
    }

    pub fn inject_batch(&self, batch: BatchApi<T>) {
        for (injector, unparker) in self.0.injectors.iter().zip(&self.0.unparkers) {
            injector.push(batch.clone());
            unparker.unpark();
        }
    }

    pub fn steal_task_from_other_workers(
        &self,
        worker_id: usize,
    ) -> Option<ScheduledTransaction<T>> {
        // TODO: randomize stealer selection
        for (id, other) in self.0.stealers.iter().enumerate() {
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
        self.0.shutdown.open();

        // wake all workers so they can exit
        for unparker in &self.0.unparkers {
            unparker.unpark();
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.0.shutdown.is_open()
    }
}

impl<T: Transaction> Clone for WorkersApi<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
