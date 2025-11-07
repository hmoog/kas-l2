use std::{hint::spin_loop, sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Steal, Stealer};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::Unparker;
use kas_l2_core_atomics::AtomicAsyncLatch;
use kas_l2_core_macros::smart_pointer;
use kas_l2_storage_manager::Store;
use tap::Tap;

use crate::{Batch, RuntimeState, RuntimeTx, VecExt, Worker, vm::VM};

#[smart_pointer]
pub(crate) struct WorkersApi<S: Store<StateSpace = RuntimeState>, V: VM> {
    worker_count: usize,
    inboxes: Vec<Arc<ArrayQueue<Batch<S, V>>>>,
    stealers: Vec<Stealer<RuntimeTx<S, V>>>,
    unparkers: Vec<Unparker>,
    shutdown: AtomicAsyncLatch,
}

impl<S: Store<StateSpace = RuntimeState>, V: VM> WorkersApi<S, V> {
    pub fn new_with_workers(worker_count: usize, vm: V) -> (Self, Vec<JoinHandle<()>>) {
        let mut data = WorkersApiData {
            worker_count,
            stealers: Vec::with_capacity(worker_count),
            unparkers: Vec::with_capacity(worker_count),
            inboxes: Vec::with_capacity(worker_count),
            shutdown: AtomicAsyncLatch::new(),
        };

        let workers: Vec<Worker<S, V>> = (0..worker_count).into_vec(|id| {
            Worker::new(id, vm.clone()).tap(|w| {
                data.inboxes.push(w.inbox());
                data.stealers.push(w.stealer());
                data.unparkers.push(w.unparker());
            })
        });

        let this = Self(Arc::new(data));
        let handles = workers.into_vec(|w| w.start(this.clone()));

        (this, handles)
    }

    pub fn push_batch(&self, batch: Batch<S, V>) {
        for (inbox, unparker) in self.inboxes.iter().zip(&self.unparkers) {
            let mut item = batch.clone();
            loop {
                match inbox.push(item) {
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

    pub fn steal_from_other_workers(&self, worker_id: usize) -> Option<RuntimeTx<S, V>> {
        if self.worker_count > 1 {
            let start = fastrand::usize(..self.worker_count);
            for offset in 0..self.worker_count {
                let id = (start + offset) % self.worker_count;
                if id != worker_id {
                    loop {
                        match self.stealers[id].steal() {
                            Steal::Success(task) => return Some(task),
                            Steal::Retry => continue,
                            Steal::Empty => break,
                        }
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
