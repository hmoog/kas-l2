use kas_l2_storage::{Storage, Store};
use tap::Tap;

use crate::{
    Batch, BatchProcessor, Executor, ResourceProvider, RuntimeBatchProcessor, RuntimeBuilder,
    Scheduler, Transaction, TransactionProcessor,
    io::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

pub struct Runtime<T: Transaction, S: Store<StateSpace = RuntimeState>> {
    storage: Storage<S, Read<T>, Write<T>>,
    scheduler: Scheduler<T>,
    executor: Executor<T>,
    batch_processor: RuntimeBatchProcessor<T>,
}

impl<T: Transaction, S: Store<StateSpace = RuntimeState>> Runtime<T, S> {
    pub fn process(&mut self, transactions: Vec<T>) -> Batch<T> {
        self.scheduler
            .schedule(&self.storage, transactions)
            .tap(|batch| {
                self.executor.execute(batch.clone());
                self.batch_processor.push(batch.clone());
            })
    }

    pub fn shutdown(self) {
        self.executor.shutdown();
        self.batch_processor.shutdown();
        self.storage.shutdown();
    }

    pub(crate) fn new<P: TransactionProcessor<T>, B: BatchProcessor<T>>(
        builder: RuntimeBuilder<T, S, P, B>,
    ) -> Self {
        let transaction_processor = builder
            .transaction_processor
            .expect("Processor must be provided before calling build()");

        Self {
            scheduler: Scheduler::new(ResourceProvider::new()),
            executor: Executor::new(builder.execution_workers, transaction_processor),
            batch_processor: RuntimeBatchProcessor::new(builder.batch_processor),
            storage: Storage::new(builder.storage_config),
        }
    }
}
