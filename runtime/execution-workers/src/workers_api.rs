use std::{hint::spin_loop, sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Steal, Stealer};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::Unparker;
use kas_l2_core_atomics::AtomicAsyncLatch;
use kas_l2_core_macros::smart_pointer;
use tap::Tap;
use tracing::{debug, trace};

use crate::{Batch, Worker, task::Task};

#[smart_pointer]
pub struct WorkersApi<T: Task, B: Batch<T>> {
    worker_count: usize,
    inboxes: Vec<Arc<ArrayQueue<B>>>,
    stealers: Vec<Stealer<T>>,
    unparkers: Vec<Unparker>,
    shutdown: AtomicAsyncLatch,
}

impl<T: Task, B: Batch<T>> WorkersApi<T, B> {
    pub fn new_with_workers(worker_count: usize) -> (Self, Vec<JoinHandle<()>>) {
        let mut data = WorkersApiData {
            worker_count,
            stealers: Vec::with_capacity(worker_count),
            unparkers: Vec::with_capacity(worker_count),
            inboxes: Vec::with_capacity(worker_count),
            shutdown: AtomicAsyncLatch::new(),
        };

        let workers: Vec<Worker<T, B>> = (0..worker_count)
            .map(|id| {
                Worker::new(id).tap(|w| {
                    data.inboxes.push(w.inbox());
                    data.stealers.push(w.stealer());
                    data.unparkers.push(w.unparker());
                })
            })
            .collect();

        let this = Self(Arc::new(data));
        let handles = workers.into_iter().map(|w| w.start(this.clone())).collect();

        (this, handles)
    }

    pub fn push_batch(&self, batch: B) {
        debug!("pushing batch to all workers");
        for (worker_id, (inbox, unparker)) in self.inboxes.iter().zip(&self.unparkers).enumerate() {
            let mut item = batch.clone();
            loop {
                match inbox.push(item) {
                    Ok(()) => {
                        trace!(worker_id, "batch pushed to inbox");
                        break;
                    }
                    Err(back) => {
                        item = back;
                        spin_loop(); // CPU relax; does NOT yield/park
                    }
                }
            }
            trace!(worker_id, "unparking worker");
            unparker.unpark();
        }
    }

    pub fn steal_from_other_workers(&self, worker_id: usize) -> Option<T> {
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

    pub fn wake_all(&self) {
        trace!("waking all workers");
        for unparker in &self.unparkers {
            unparker.unpark();
        }
    }
}
