use std::{sync::Arc, thread, thread::JoinHandle, time::Duration};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};
use kas_l2_storage::Store;

use crate::{
    Batch, PendingBatches, RuntimeState, RuntimeTx, Transaction, TransactionProcessor, WorkersApi,
};

pub struct Worker<
    S: Store<StateSpace = RuntimeState>,
    T: Transaction,
    P: TransactionProcessor<S, T>,
> {
    id: usize,
    local_queue: WorkerQueue<RuntimeTx<S, T>>,
    inbox: Arc<ArrayQueue<Batch<S, T>>>,
    processor: P,
    parker: Parker,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction, P: TransactionProcessor<S, T>>
    Worker<S, T, P>
{
    pub(crate) fn new(id: usize, processor: P) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            inbox: Arc::new(ArrayQueue::new(1024)),
            processor,
            parker: Parker::new(),
        }
    }

    pub(crate) fn start(self, workers_api: WorkersApi<S, T>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    pub(crate) fn stealer(&self) -> Stealer<RuntimeTx<S, T>> {
        self.local_queue.stealer()
    }

    pub(crate) fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub(crate) fn inbox(&self) -> Arc<ArrayQueue<Batch<S, T>>> {
        self.inbox.clone()
    }

    fn run(self, workers_api: WorkersApi<S, T>) {
        let mut pending_batches = PendingBatches::new(self.inbox);

        while !workers_api.is_shutdown() {
            match self
                .local_queue
                .pop()
                .or_else(|| pending_batches.steal(&self.local_queue))
                .or_else(|| workers_api.steal_from_other_workers(self.id))
            {
                Some(task) => task.execute(&self.processor),
                None => self.parker.park_timeout(Duration::from_millis(100)),
            }
        }
    }
}
