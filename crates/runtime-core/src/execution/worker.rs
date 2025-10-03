use std::{sync::Arc, thread, thread::JoinHandle};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};

use crate::{BatchApi, PendingBatches, RuntimeTx, Transaction, TransactionProcessor, WorkersApi};

pub struct Worker<T: Transaction, P: TransactionProcessor<T>> {
    id: usize,
    local_queue: WorkerQueue<RuntimeTx<T>>,
    injector: Arc<ArrayQueue<BatchApi<T>>>,
    processor: P,
    parker: Parker,
}

impl<T: Transaction, P: TransactionProcessor<T>> Worker<T, P> {
    pub(crate) fn new(id: usize, processor: P) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            injector: Arc::new(ArrayQueue::new(1024)),
            processor,
            parker: Parker::new(),
        }
    }

    pub(crate) fn start(self, workers_api: WorkersApi<T>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    pub(crate) fn stealer(&self) -> Stealer<RuntimeTx<T>> {
        self.local_queue.stealer()
    }

    pub(crate) fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub(crate) fn injector(&self) -> Arc<ArrayQueue<BatchApi<T>>> {
        self.injector.clone()
    }

    fn run(self, workers_api: WorkersApi<T>) {
        let mut pending_batches = PendingBatches::new(self.injector);

        while !workers_api.is_shutdown() {
            match self
                .local_queue
                .pop()
                .or_else(|| pending_batches.steal(&self.local_queue))
                .or_else(|| workers_api.steal_from_other_workers(self.id))
            {
                Some(task) => task.execute(&self.processor),
                None => self.parker.park(),
            }
        }
    }
}
