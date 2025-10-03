use crate::{
    BatchApi, BatchProcessor, Executor, ResourceProvider, RuntimeBatchProcessor, RuntimeBuilder,
    Scheduler, Storage, Transaction, TransactionProcessor,
};

pub struct Runtime<T: Transaction, S: Storage<T::ResourceID>> {
    scheduler: Scheduler<T, S>,
    executor: Executor<T>,
    batch_processor: RuntimeBatchProcessor<T>,
}

impl<T: Transaction, S: Storage<T::ResourceID>> Runtime<T, S> {
    pub fn process(&mut self, transactions: Vec<T>) -> BatchApi<T> {
        let batch = self.scheduler.schedule(transactions);
        let batch_api = batch.api().clone();

        self.executor.execute(batch_api.clone());
        self.batch_processor.push(batch);

        batch_api
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
