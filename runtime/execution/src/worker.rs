use std::{sync::Arc, thread, thread::JoinHandle, time::Duration};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};

use crate::{ExecutionBatchQueue, ExecutionTask, WorkersApi};

pub struct Worker<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    id: usize,
    local_queue: WorkerQueue<T>,
    inbox: Arc<ArrayQueue<Q::Batch>>,
    vm: V,
    parker: Parker,
}

impl<V, T, Q> Worker<V, T, Q>
where
    T: ExecutionTask<V> + Send + 'static,
    V: Clone + Send + Sync + 'static,
    Q: ExecutionBatchQueue<T> + 'static,
{
    pub(crate) fn new(id: usize, vm: V) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            inbox: Arc::new(ArrayQueue::new(1024)),
            vm,
            parker: Parker::new(),
        }
    }

    pub(crate) fn start(self, workers_api: WorkersApi<V, T, Q>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    pub(crate) fn stealer(&self) -> Stealer<T> {
        self.local_queue.stealer()
    }

    pub(crate) fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub(crate) fn inbox(&self) -> Arc<ArrayQueue<Q::Batch>> {
        self.inbox.clone()
    }

    fn run(self, workers_api: WorkersApi<V, T, Q>) {
        let mut pending_batches = Q::new(self.inbox);

        while !workers_api.is_shutdown() {
            match self
                .local_queue
                .pop()
                .or_else(|| pending_batches.steal(&self.local_queue))
                .or_else(|| workers_api.steal_from_other_workers(self.id))
            {
                Some(task) => task.execute(&self.vm),
                None => self.parker.park_timeout(Duration::from_millis(100)),
            }
        }
    }
}
