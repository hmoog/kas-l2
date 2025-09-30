use std::sync::Arc;

use crate::{
    BatchAPI, RuntimeBuilder, Storage, Transaction, TransactionProcessor,
    execution::executor::Executor, resources::resource_provider::ResourceProvider,
    scheduling::scheduler::Scheduler,
};

pub struct Runtime<T: Transaction, S: Storage<T::ResourceID>> {
    scheduler: Scheduler<T, S>,
    executor: Executor<T>,
}

impl<T: Transaction, S: Storage<T::ResourceID>> Runtime<T, S> {
    pub fn process(&mut self, transactions: Vec<T>) -> Arc<BatchAPI<T>> {
        let batch_api = self.scheduler.schedule(transactions);
        self.executor.execute(batch_api.clone());
        batch_api
    }

    pub fn shutdown(self) {
        self.executor.shutdown();
    }

    pub(crate) fn new<P: TransactionProcessor<T>>(builder: RuntimeBuilder<T, S, P>) -> Self {
        let storage = builder
            .storage
            .expect("Storage must be provided before calling build()");

        let processor = builder
            .transaction_processor
            .expect("Processor must be provided before calling build()");

        Self {
            scheduler: Scheduler::new(ResourceProvider::new(storage)),
            executor: Executor::new(builder.execution_workers, processor),
        }
    }
}
