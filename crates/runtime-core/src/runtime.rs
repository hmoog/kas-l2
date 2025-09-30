use std::{sync::Arc, thread, thread::JoinHandle};

use crossbeam_queue::SegQueue;
use tokio::{runtime::Builder, sync::Notify};

use crate::{
    BatchAPI, BatchProcessor, RuntimeBuilder, Storage, Transaction, TransactionProcessor,
    execution::executor::Executor,
    resources::resource_provider::ResourceProvider,
    scheduling::{batch::Batch, scheduler::Scheduler},
};

pub struct Runtime<T: Transaction, S: Storage<T::ResourceID>> {
    scheduler: Scheduler<T, S>,
    executor: Executor<T>,
    batch_processor: JoinHandle<()>,
    batch_processor_queue: Arc<SegQueue<Batch<T>>>,
    batch_processor_waker: Arc<Notify>,
}

impl<T: Transaction, S: Storage<T::ResourceID>> Runtime<T, S> {
    pub fn process(&mut self, transactions: Vec<T>) -> Arc<BatchAPI<T>> {
        let batch = self.scheduler.schedule(transactions);
        let batch_api = batch.api().clone();

        self.executor.execute(batch_api.clone());
        self.batch_processor_queue.push(batch);
        self.batch_processor_waker.notify_one();

        batch_api
    }

    pub fn shutdown(self) {
        self.executor.shutdown();

        drop(self.batch_processor_queue);
        self.batch_processor_waker.notify_one();
        let _ = self.batch_processor.join();
    }

    pub(crate) fn new<P: TransactionProcessor<T>, B: BatchProcessor<T>>(
        builder: RuntimeBuilder<T, S, P, B>,
    ) -> Self {
        let storage = builder
            .storage
            .expect("Storage must be provided before calling build()");

        let processor = builder
            .transaction_processor
            .expect("Processor must be provided before calling build()");

        let queue = Arc::new(SegQueue::new());
        let notify = Arc::new(Notify::new());
        let batch_processor_worker =
            Self::spawn_batch_processor(queue.clone(), notify.clone(), builder.batch_processor);

        Self {
            scheduler: Scheduler::new(ResourceProvider::new(storage)),
            executor: Executor::new(builder.execution_workers, processor),
            batch_processor: batch_processor_worker,
            batch_processor_queue: queue,
            batch_processor_waker: notify,
        }
    }

    /// Internal helper: spawn the worker thread and run the async loop inside it.
    fn spawn_batch_processor<F: BatchProcessor<T>>(
        queue: Arc<SegQueue<Batch<T>>>,
        notify: Arc<Notify>,
        callback: F,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            Builder::new_current_thread()
                .build()
                .expect("failed to build Tokio runtime for worker")
                .block_on(async move {
                    loop {
                        // Drain all available batches
                        while let Some(batch) = queue.pop() {
                            // Strict FIFO: wait until batch completes
                            batch.api().wait_done().await;

                            // Call user-provided callback
                            callback(batch);
                        }

                        // If no producers left, exit
                        if Arc::strong_count(&queue) == 1 {
                            break;
                        }

                        // Otherwise wait to be notified
                        notify.notified().await;
                    }
                });
        })
    }
}
