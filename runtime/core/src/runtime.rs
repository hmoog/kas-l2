use kas_l2_storage_manager::{StorageConfig, StorageManager, Store};
use tap::Tap;

use crate::{
    Batch, Executor, NotarizationWorker, Notarizer, Read, Scheduler, Transaction,
    TransactionProcessor, Write, storage::runtime_state::RuntimeState,
};

pub struct Runtime<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    storage: StorageManager<S, Read<S, T>, Write<S, T>>,
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

    pub fn from_parts<P: TransactionProcessor<S, T>, B: Notarizer<S, T>>(
        execution_workers: usize,
        transaction_processor: P,
        notarizer: B,
        storage_config: StorageConfig<S>,
    ) -> Self {
        let storage = StorageManager::new(storage_config);

        Self {
            scheduler: Scheduler::new(storage.clone()),
            executor: Executor::new(execution_workers, transaction_processor),
            notarization: NotarizationWorker::new(notarizer),
            storage,
        }
    }
}
