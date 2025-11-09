use kas_l2_runtime_state_space::StateSpace;
use kas_l2_runtime_storage_manager::{StorageConfig, StorageManager};
use kas_l2_storage_interface::Store;
use tap::Tap;

use crate::{Batch, Executor, NotarizationWorker, Read, Scheduler, Write, vm::VM};

pub struct Runtime<S: Store<StateSpace = StateSpace>, V: VM> {
    storage: StorageManager<S, Read<S, V>, Write<S, V>>,
    scheduler: Scheduler<S, V>,
    executor: Executor<S, V>,
    notarization: NotarizationWorker<S, V>,
}

impl<S: Store<StateSpace = StateSpace>, V: VM> Runtime<S, V> {
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

    pub fn from_parts(execution_workers: usize, vm: V, storage_config: StorageConfig<S>) -> Self {
        let storage = StorageManager::new(storage_config);

        Self {
            scheduler: Scheduler::new(storage.clone()),
            executor: Executor::new(execution_workers, vm.clone()),
            notarization: NotarizationWorker::new(vm),
            storage,
        }
    }
}
