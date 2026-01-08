use std::{sync::Arc, thread, thread::JoinHandle, time::Duration};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};
use tracing::{debug, trace};

use crate::{Batch, BatchQueue, WorkersApi, task::Task};

pub struct Worker<T: Task, B: Batch<T>> {
    id: usize,
    local_queue: WorkerQueue<T>,
    inbox: Arc<ArrayQueue<B>>,
    parker: Parker,
}

impl<T: Task, B: Batch<T>> Worker<T, B> {
    pub(crate) fn new(id: usize) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            inbox: Arc::new(ArrayQueue::new(1024)),
            parker: Parker::new(),
        }
    }

    pub(crate) fn start(self, workers_api: WorkersApi<T, B>) -> JoinHandle<()> {
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

    fn run(self, workers_api: WorkersApi<T, B>) {
        debug!(worker_id = self.id, "worker started");
        let mut pending_batches = BatchQueue::new(self.inbox);

        while !workers_api.is_shutdown() {
            let from_local = self.local_queue.pop();
            let from_batches = from_local.is_none().then(|| pending_batches.steal(&self.local_queue)).flatten();
            let from_steal = from_batches.is_none().then(|| workers_api.steal_from_other_workers(self.id)).flatten();

            match from_local.or(from_batches).or(from_steal) {
                Some(task) => {
                    trace!(worker_id = self.id, "executing task");
                    task.execute();
                }
                None => {
                    trace!(worker_id = self.id, "no work found, parking");
                    self.parker.park_timeout(Duration::from_millis(100));
                    trace!(worker_id = self.id, "worker unparked");
                }
            }
        }
        debug!(worker_id = self.id, "worker shutdown");
    }
}
