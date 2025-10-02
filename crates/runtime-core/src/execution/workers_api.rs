use std::{sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Injector, Steal, Stealer};
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_runtime_macros::smart_pointer;
use tap::Tap;

use crate::{
    BatchApi, RuntimeTx, Transaction, TransactionProcessor, execution::worker::Worker,
    utils::vec_ext::VecExt,
};

#[smart_pointer]
pub(crate) struct WorkersApi<T: Transaction> {
    stealers: Vec<Stealer<RuntimeTx<T>>>,
    unparkers: Vec<Unparker>,
    injectors: Vec<Arc<Injector<BatchApi<T>>>>,
    shutdown: AtomicAsyncLatch,
}

impl<T: Transaction> WorkersApi<T> {
    pub fn new_with_workers<P: TransactionProcessor<T>>(
        worker_count: usize,
        processor: P,
    ) -> (Self, Vec<JoinHandle<()>>) {
        let mut data = WorkersApiData {
            stealers: Vec::with_capacity(worker_count),
            unparkers: Vec::with_capacity(worker_count),
            injectors: Vec::with_capacity(worker_count),
            shutdown: AtomicAsyncLatch::new(),
        };

        let workers: Vec<Worker<T, P>> = (0..worker_count).into_vec(|id| {
            Worker::new(id, processor.clone()).tap(|w| {
                data.stealers.push(w.stealer());
                data.unparkers.push(w.unparker());
                data.injectors.push(w.injector());
            })
        });

        let this = Self(Arc::new(data));
        let handles = workers.into_vec(|w| w.start(this.clone()));

        (this, handles)
    }

    pub fn inject_batch(&self, batch: BatchApi<T>) {
        for (injector, unparker) in self.0.injectors.iter().zip(&self.0.unparkers) {
            injector.push(batch.clone());
            unparker.unpark();
        }
    }

    pub fn steal_task_from_other_workers(&self, worker_id: usize) -> Option<RuntimeTx<T>> {
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
        self.0.shutdown.open(); // trigger shutdown signal

        for unparker in &self.0.unparkers {
            unparker.unpark();
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.0.shutdown.is_open()
    }
}
