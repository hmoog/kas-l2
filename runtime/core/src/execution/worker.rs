use std::{sync::Arc, thread, thread::JoinHandle, time::Duration};

use crossbeam_deque::{Stealer, Worker as WorkerQueue};
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::sync::{Parker, Unparker};
use kas_l2_storage_manager::Store;

use crate::{Batch, BatchQueue, RuntimeState, RuntimeTx, Vm, WorkersApi};

pub struct Worker<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    id: usize,
    local_queue: WorkerQueue<RuntimeTx<S, VM>>,
    inbox: Arc<ArrayQueue<Batch<S, VM>>>,
    vm: VM,
    parker: Parker,
}

impl<S, VM> Worker<S, VM>
where
    S: Store<StateSpace = RuntimeState>,
    VM: Vm,
{
    pub(crate) fn new(id: usize, vm: VM) -> Self {
        Self {
            id,
            local_queue: WorkerQueue::new_fifo(),
            inbox: Arc::new(ArrayQueue::new(1024)),
            vm,
            parker: Parker::new(),
        }
    }

    pub(crate) fn start(self, workers_api: WorkersApi<S, VM>) -> JoinHandle<()> {
        thread::spawn(move || self.run(workers_api))
    }

    pub(crate) fn stealer(&self) -> Stealer<RuntimeTx<S, VM>> {
        self.local_queue.stealer()
    }

    pub(crate) fn unparker(&self) -> Unparker {
        self.parker.unparker().clone()
    }

    pub(crate) fn inbox(&self) -> Arc<ArrayQueue<Batch<S, VM>>> {
        self.inbox.clone()
    }

    fn run(self, workers_api: WorkersApi<S, VM>) {
        let mut pending_batches = BatchQueue::new(self.inbox);

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
