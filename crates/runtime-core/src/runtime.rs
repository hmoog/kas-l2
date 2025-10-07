use tap::Tap;

use crate::{
    Batch, BatchProcessor, Executor, ResourceProvider, RuntimeBatchProcessor, RuntimeBuilder,
    Scheduler, Storage, Transaction, TransactionProcessor,
};

pub struct Runtime<T: Transaction, S: Storage<T::ResourceId>> {
    scheduler: Scheduler<T, S>,
    executor: Executor<T>,
    batch_processor: RuntimeBatchProcessor<T>,
}

impl<T: Transaction, S: Storage<T::ResourceId>> Runtime<T, S> {
    pub fn process(&mut self, transactions: Vec<T>) -> Batch<T> {
        self.scheduler.schedule(transactions).tap(|batch| {
            self.executor.execute(batch.clone());
            self.batch_processor.push(batch.clone());
        })
    }

    pub fn shutdown(self) {
        self.executor.shutdown();
        self.batch_processor.shutdown();
    }

    pub(crate) fn new<P: TransactionProcessor<T>, B: BatchProcessor<T>>(
        builder: RuntimeBuilder<T, S, P, B>,
    ) -> Self {
        let storage = builder
            .storage
            .expect("Storage must be provided before calling build()");

        let transaction_processor = builder
            .transaction_processor
            .expect("Processor must be provided before calling build()");

        Self {
            scheduler: Scheduler::new(ResourceProvider::new(storage)),
            executor: Executor::new(builder.execution_workers, transaction_processor),
            batch_processor: RuntimeBatchProcessor::new(builder.batch_processor),
        }
    }
}
