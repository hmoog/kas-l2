use kas_l2_io_manager::{IoManager, Storage};
use tap::Tap;

use crate::{
    Batch, BatchProcessor, Executor, ResourceProvider, RuntimeBatchProcessor, RuntimeBuilder,
    Scheduler, Transaction, TransactionProcessor,
    io::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

pub struct Runtime<T: Transaction, S: Storage<Namespace = RuntimeState>> {
    io: IoManager<S, Read<T>, Write<T>>,
    scheduler: Scheduler<T>,
    executor: Executor<T>,
    batch_processor: RuntimeBatchProcessor<T>,
}

impl<T: Transaction, S: Storage<Namespace = RuntimeState>> Runtime<T, S> {
    pub fn process(&mut self, transactions: Vec<T>) -> Batch<T> {
        self.scheduler
            .schedule(&self.io, transactions)
            .tap(|batch| {
                self.executor.execute(batch.clone());
                self.batch_processor.push(batch.clone());
            })
    }

    pub fn shutdown(self) {
        self.executor.shutdown();
        self.batch_processor.shutdown();
        self.io.shutdown();
    }

    pub(crate) fn new<P: TransactionProcessor<T>, B: BatchProcessor<T>>(
        builder: RuntimeBuilder<T, S, P, B>,
    ) -> Self {
        let storage = builder
            .store
            .expect("Storage must be provided before calling build()");

        let transaction_processor = builder
            .transaction_processor
            .expect("Processor must be provided before calling build()");

        let io = IoManager::new(storage, builder.io_config);

        Self {
            scheduler: Scheduler::new(ResourceProvider::new()),
            executor: Executor::new(builder.execution_workers, transaction_processor),
            batch_processor: RuntimeBatchProcessor::new(builder.batch_processor),
            io,
        }
    }
}
