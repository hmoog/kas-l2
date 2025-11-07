use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_queue::SegQueue;
use kas_l2_storage_manager::Store;
use tokio::{runtime::Builder, sync::Notify};

use crate::{Batch, RuntimeState, Vm};

pub(crate) struct NotarizationWorker<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    queue: Arc<SegQueue<Batch<S, VM>>>,
    notify: Arc<Notify>,
    handle: JoinHandle<()>,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> NotarizationWorker<S, VM> {
    pub(crate) fn new(vm: VM) -> Self {
        let queue = Arc::new(SegQueue::new());
        let notify = Arc::new(Notify::new());
        let handle = Self::start(queue.clone(), notify.clone(), vm);

        Self { queue, notify, handle }
    }

    pub(crate) fn push(&self, batch: Batch<S, VM>) {
        self.queue.push(batch);
        self.notify.notify_one();
    }

    pub(crate) fn shutdown(self) {
        drop(self.queue);
        self.notify.notify_one();
        self.handle.join().expect("batch processor panicked");
    }

    fn start(queue: Arc<SegQueue<Batch<S, VM>>>, notify: Arc<Notify>, vm: VM) -> JoinHandle<()> {
        thread::spawn(move || {
            Builder::new_current_thread().build().expect("failed to build tokio runtime").block_on(
                async move {
                    while Arc::strong_count(&queue) != 1 {
                        while let Some(batch) = queue.pop() {
                            batch.wait_processed().await;
                            vm.notarize(&batch);
                            batch.wait_persisted().await;
                            batch.schedule_commit();
                            batch.wait_committed().await;
                        }

                        if Arc::strong_count(&queue) != 1 {
                            notify.notified().await
                        }
                    }
                },
            )
        })
    }
}
