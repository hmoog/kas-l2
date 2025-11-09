use std::{sync::Arc, thread, thread::JoinHandle, time::Duration};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};

use crate::{BatchQueue, ExecutionTask, TaskBatch, WorkersApi};

pub struct Worker<T, B, V>
where
    T: ExecutionTask<V> + Clone + Send + 'static,
    B: TaskBatch<T>,
    V: Clone + Send + Sync + 'static,
{
    id: usize,
    local_queue: WorkerQueue<T>,
    inbox: Arc<ArrayQueue<B>>,
    vm: V,
    parker: Parker,
}

impl<T, B, V> Worker<T, B, V>
where
    T: ExecutionTask<V> + Clone + Send + 'static,
    B: TaskBatch<T>,
    V: Clone + Send + Sync + 'static,
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

    pub(crate) fn start(self, workers_api: WorkersApi<T, B, V>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    pub(crate) fn stealer(&self) -> Stealer<T> {
        self.local_queue.stealer()
    }

    pub(crate) fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub(crate) fn inbox(&self) -> Arc<ArrayQueue<B>> {
        self.inbox.clone()
    }

    fn run(self, workers_api: WorkersApi<T, B, V>) {
        let mut pending_batches = BatchQueue::new(self.inbox);

        while !workers_api.is_shutdown() {
            match self
                .local_queue
                .pop()
                .or_else(|| pending_batches.steal(&self.local_queue))
                .or_else(|| workers_api.steal_from_other_workers(self.id))
            {
                Some(task) => task.execute_with(&self.vm),
                None => self.parker.park_timeout(Duration::from_millis(100)),
            }
        }
    }
}
