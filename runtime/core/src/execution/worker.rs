use std::{sync::Arc, thread, thread::JoinHandle, time::Duration};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};
use kas_l2_storage_manager::Store;

use crate::{Batch, BatchQueue, RuntimeState, RuntimeTx, TransactionProcessor, WorkersApi, vm::VM};

pub struct Worker<S: Store<StateSpace = RuntimeState>, V: VM, P: TransactionProcessor<S, V>> {
    id: usize,
    local_queue: WorkerQueue<RuntimeTx<S, V>>,
    inbox: Arc<ArrayQueue<Batch<S, V>>>,
    processor: P,
    parker: Parker,
}

impl<S, V, P> Worker<S, V, P>
where
    S: Store<StateSpace = RuntimeState>,
    V: VM,
    P: TransactionProcessor<S, V>,
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

    pub(crate) fn start(self, workers_api: WorkersApi<S, V>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    pub(crate) fn stealer(&self) -> Stealer<RuntimeTx<S, V>> {
        self.local_queue.stealer()
    }

    pub(crate) fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub(crate) fn inbox(&self) -> Arc<ArrayQueue<Batch<S, V>>> {
        self.inbox.clone()
    }

    fn run(self, workers_api: WorkersApi<S, V>) {
        let mut pending_batches = BatchQueue::new(self.inbox);

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
