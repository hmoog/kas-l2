use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_queue::SegQueue;
use kas_l2_storage::Store;
use tokio::{runtime::Builder, sync::Notify};

use crate::{Batch, BatchProcessor, RuntimeState, Transaction};

pub(crate) struct RuntimeBatchProcessor<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    queue: Arc<SegQueue<Batch<S, T>>>,
    notify: Arc<Notify>,
    handle: JoinHandle<()>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> RuntimeBatchProcessor<S, T> {
    pub(crate) fn new<B: BatchProcessor<S, T>>(batch_processor: B) -> Self {
        let queue = Arc::new(SegQueue::new());
        let notify = Arc::new(Notify::new());
        let handle = Self::start(queue.clone(), notify.clone(), batch_processor);

        Self {
            queue,
            notify,
            handle,
        }
    }

    pub(crate) fn push(&self, batch: Batch<S, T>) {
        self.queue.push(batch);
        self.notify.notify_one();
    }

    pub(crate) fn shutdown(self) {
        drop(self.queue);
        self.notify.notify_one();
        self.handle.join().expect("batch processor panicked");
    }

    fn start<F: BatchProcessor<S, T>>(
        queue: Arc<SegQueue<Batch<S, T>>>,
        notify: Arc<Notify>,
        batch_processor: F,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            Builder::new_current_thread()
                .build()
                .expect("failed to build Tokio runtime")
                .block_on(async move {
                    while Arc::strong_count(&queue) != 1 {
                        while let Some(batch) = queue.pop() {
                            batch.wait_processed().await;
                            batch_processor(batch);
                        }

                        if Arc::strong_count(&queue) != 1 {
                            notify.notified().await
                        }
                    }
                })
        })
    }
}
