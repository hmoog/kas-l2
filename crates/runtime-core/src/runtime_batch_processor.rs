use std::sync::Arc;
use std::thread::{self, JoinHandle};
use crossbeam_queue::SegQueue;
use tokio::runtime::Builder;
use tokio::sync::Notify;

use crate::{Batch, BatchProcessor, Transaction};

pub(crate) struct RuntimeBatchProcessor<T: Transaction> {
    queue: Arc<SegQueue<Batch<T>>>,
    notify: Arc<Notify>,
    handle: JoinHandle<()>,
}

impl<T: Transaction> RuntimeBatchProcessor<T> {
    pub(crate) fn new<F: BatchProcessor<T>>(batch_processor: F) -> Self {
        let queue = Arc::new(SegQueue::new());
        let notify = Arc::new(Notify::new());
        let handle = Self::spawn(queue.clone(), notify.clone(), batch_processor);

        Self { queue, notify, handle }
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

    fn spawn<F: BatchProcessor<T>>(queue: Arc<SegQueue<Batch<T>>>, notify: Arc<Notify>, callback: F) -> JoinHandle<()> {
        thread::spawn(move || {
            let rt = Builder::new_current_thread()
                .build()
                .expect("failed to build Tokio runtime");

            rt.block_on(async move {
                while Arc::strong_count(&queue) != 1 {
                    while let Some(batch) = queue.pop() {
                        batch.api().wait_done().await;
                        callback(batch);
                    }

                    match Arc::strong_count(&queue) {
                        1 => break,
                        _ => notify.notified().await,
                    }
                }
            });
        })
    }
}
