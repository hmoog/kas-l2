use std::{hint::spin_loop, marker::PhantomData, sync::Arc, thread::JoinHandle};

use crossbeam_deque::{Steal, Stealer};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::Unparker;
use kas_l2_core_atomics::AtomicAsyncLatch;
use tap::Tap;

use crate::{ExecutionBatchQueue, ExecutionTask, Worker};

pub struct WorkersApi<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    inner: Arc<WorkersApiData<V, T, Q>>,
}

struct WorkersApiData<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    worker_count: usize,
    inboxes: Vec<Arc<ArrayQueue<Q::Batch>>>,
    stealers: Vec<Stealer<T>>,
    unparkers: Vec<Unparker>,
    shutdown: AtomicAsyncLatch,
    marker: PhantomData<V>,
}

impl<V, T, Q> Clone for WorkersApi<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<V, T, Q> WorkersApi<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    pub fn new_with_workers(worker_count: usize, vm: V) -> (Self, Vec<JoinHandle<()>>) {
        let mut data = WorkersApiData {
            worker_count,
            stealers: Vec::with_capacity(worker_count),
            unparkers: Vec::with_capacity(worker_count),
            inboxes: Vec::with_capacity(worker_count),
            shutdown: AtomicAsyncLatch::new(),
            marker: PhantomData,
        };

        let workers: Vec<Worker<V, T, Q>> = (0..worker_count)
            .map(|id| {
                Worker::new(id, vm.clone()).tap(|w| {
                    data.inboxes.push(w.inbox());
                    data.stealers.push(w.stealer());
                    data.unparkers.push(w.unparker());
                })
            })
            .collect();

        let this = Self { inner: Arc::new(data) };
        let handles = workers.into_iter().map(|w| w.start(this.clone())).collect();

        (this, handles)
    }

    pub fn push_batch(&self, batch: Q::Batch) {
        for (inbox, unparker) in self.inner.inboxes.iter().zip(&self.inner.unparkers) {
            let mut item = batch.clone();
            loop {
                match inbox.push(item) {
                    Ok(()) => break,
                    Err(back) => {
                        item = back;
                        spin_loop();
                    }
                }
            }
            unparker.unpark();
        }
    }

    pub fn steal_from_other_workers(&self, worker_id: usize) -> Option<T> {
        if self.inner.worker_count > 1 {
            let start = fastrand::usize(..self.inner.worker_count);
            for offset in 0..self.inner.worker_count {
                let id = (start + offset) % self.inner.worker_count;
                if id != worker_id {
                    loop {
                        match self.inner.stealers[id].steal() {
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
        self.inner.shutdown.open();

        for unparker in &self.inner.unparkers {
            unparker.unpark();
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.inner.shutdown.is_open()
    }
}
