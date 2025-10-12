use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_queue::SegQueue;
use tokio::{runtime::Builder, sync::Notify};

use crate::{Batch, BatchProcessor, Transaction};

pub(crate) struct RuntimeBatchProcessor<T: Transaction> {
    queue: Arc<SegQueue<Batch<T>>>,
    notify: Arc<Notify>,
    handle: JoinHandle<()>,
}

impl<T: Transaction> RuntimeBatchProcessor<T> {
    pub(crate) fn new<B: BatchProcessor<T>>(batch_processor: B) -> Self {
        let queue = Arc::new(SegQueue::new());
        let notify = Arc::new(Notify::new());
        let handle = Self::start(queue.clone(), notify.clone(), batch_processor);

        Self {
            queue,
            notify,
            handle,
        }
    }

    pub(crate) fn push(&self, batch: Batch<T>) {
        self.queue.push(batch);
        self.notify.notify_one();
    }

    pub(crate) fn shutdown(self) {
        drop(self.queue);
        self.notify.notify_one();
        self.handle.join().expect("batch processor panicked");
    }

    fn start<F: BatchProcessor<T>>(
        queue: Arc<SegQueue<Batch<T>>>,
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
