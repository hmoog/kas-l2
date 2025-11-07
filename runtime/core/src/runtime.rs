use kas_l2_storage_manager::{StorageConfig, StorageManager, Store};
use tap::Tap;

use crate::{
    Batch, Executor, NotarizationWorker, Notarizer, Read, Scheduler, Write,
    storage::runtime_state::RuntimeState, vm::VM,
};

pub struct Runtime<S: Store<StateSpace = RuntimeState>, V: VM> {
    storage: StorageManager<S, Read<S, V>, Write<S, V>>,
    scheduler: Scheduler<S, V>,
    executor: Executor<S, V>,
    notarization: NotarizationWorker<S, V>,
}

impl<S: Store<StateSpace = RuntimeState>, V: VM> Runtime<S, V> {
    pub fn process(&mut self, transactions: Vec<V::Transaction>) -> Batch<S, V> {
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

    pub fn from_parts<B: Notarizer<S, V>>(
        execution_workers: usize,
        vm: V,
        notarizer: B,
        storage_config: StorageConfig<S>,
    ) -> Self {
        let storage = StorageManager::new(storage_config);

        Self {
            scheduler: Scheduler::new(storage.clone()),
            executor: Executor::new(execution_workers, vm),
            notarization: NotarizationWorker::new(notarizer),
            storage,
        }
    }
}
