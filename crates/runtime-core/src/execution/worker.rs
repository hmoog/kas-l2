use std::{sync::Arc, thread, thread::JoinHandle};

use crossbeam_deque::{Injector, Stealer, Worker as WorkerQueue};
use crossbeam_utils::sync::{Parker, Unparker};

use crate::{
    BatchApi, RuntimeTx, Transaction, TransactionProcessor,
    execution::{batch_injector::BatchInjector, workers_api::WorkersApi},
};

pub struct Worker<T: Transaction, P: TransactionProcessor<T>> {
    id: usize,
    local_queue: WorkerQueue<RuntimeTx<T>>,
    injector: Arc<Injector<BatchApi<T>>>,
    processor: P,
    parker: Parker,
}

impl<T: Transaction, P: TransactionProcessor<T>> Worker<T, P> {
    pub(crate) fn new(id: usize, processor: P) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            injector: Arc::new(Injector::new()),
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

    pub(crate) fn injector(&self) -> Arc<Injector<BatchApi<T>>> {
        self.injector.clone()
    }

    fn run(self, workers_api: WorkersApi<T>) {
        let mut batch_injector = BatchInjector::new(self.injector);

        while !workers_api.is_shutdown() {
            match self
                .local_queue
                .pop()
                .or_else(|| batch_injector.steal(&self.local_queue))
                .or_else(|| workers_api.steal_task_from_other_workers(self.id))
            {
                Some(task) => task.process(&self.processor),
                None => self.parker.park(),
            }
        }
    }
}
