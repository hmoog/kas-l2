use std::{hint::spin_loop, sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Steal, Stealer};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::Unparker;
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_runtime_macros::smart_pointer;
use tap::Tap;

use crate::{BatchApi, RuntimeTx, Transaction, TransactionProcessor, VecExt, Worker};

#[smart_pointer]
pub(crate) struct WorkersApi<Tx: Transaction> {
    stealers: Vec<Stealer<RuntimeTx<Tx>>>,
    unparkers: Vec<Unparker>,
    injectors: Vec<Arc<ArrayQueue<BatchApi<Tx>>>>,
    shutdown: AtomicAsyncLatch,
}

impl<Tx: Transaction> WorkersApi<Tx> {
    pub fn new_with_workers<TxProc: TransactionProcessor<Tx>>(
        worker_count: usize,
        processor: TxProc,
    ) -> (Self, Vec<JoinHandle<()>>) {
        let mut data = WorkersApiData {
            stealers: Vec::with_capacity(worker_count),
            unparkers: Vec::with_capacity(worker_count),
            injectors: Vec::with_capacity(worker_count),
            shutdown: AtomicAsyncLatch::new(),
        };

        let workers: Vec<Worker<Tx, TxProc>> = (0..worker_count).into_vec(|id| {
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

    pub fn inject_batch(&self, batch: BatchApi<Tx>) {
        for (injector, unparker) in self.injectors.iter().zip(&self.unparkers) {
            let mut item = batch.clone();
            loop {
                match injector.push(item) {
                    Ok(()) => break,
                    Err(back) => {
                        item = back;
                        spin_loop(); // CPU relax; does NOT yield/park
                    }
                }
            }
            unparker.unpark();
        }
    }

    pub fn steal_from_other_workers(&self, worker_id: usize) -> Option<RuntimeTx<Tx>> {
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
        self.shutdown.open(); // trigger shutdown signal

        for unparker in &self.unparkers {
            unparker.unpark();
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown.is_open()
    }
}
