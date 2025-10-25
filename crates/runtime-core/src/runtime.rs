use kas_l2_storage::{Storage, Store};
use tap::Tap;

use crate::{
    Batch, Executor, NotarizationWorker, Notarizer, Read, RuntimeBuilder, Scheduler, Transaction,
    TransactionProcessor, Write, storage::runtime_state::RuntimeState,
};

pub struct Runtime<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    storage: Storage<S, Read<S, T>, Write<S, T>>,
    scheduler: Scheduler<S, T>,
    executor: Executor<S, T>,
    notarization: NotarizationWorker<S, T>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> Runtime<S, T> {
    pub fn process(&mut self, transactions: Vec<T>) -> Batch<S, T> {
        self.scheduler.schedule(transactions).tap(|batch| {
            self.executor.execute(batch.clone());
            self.notarization.push(batch.clone());
        })
    }

    pub fn shutdown(self) {
        self.executor.shutdown();
        self.notarization.shutdown();
        self.storage.shutdown();
    }

    pub(crate) fn new<P: TransactionProcessor<S, T>, B: Notarizer<S, T>>(
        builder: RuntimeBuilder<T, S, P, B>,
    ) -> Self {
        let storage = Storage::new(builder.storage_config);
        let transaction_processor = builder
            .transaction_processor
            .expect("Processor must be provided before calling build()");

        Self {
            scheduler: Scheduler::new(storage.clone()),
            executor: Executor::new(builder.execution_workers, transaction_processor),
            notarization: NotarizationWorker::new(builder.notarizer),
            storage,
        }
    }
}
