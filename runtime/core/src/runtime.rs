use kas_l2_storage_manager::{StorageConfig, StorageManager, Store};
use tap::Tap;

use crate::{
    Batch, Executor, NotarizationWorker, Read, Scheduler, Vm, Write,
    storage::runtime_state::RuntimeState,
};

pub struct Runtime<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    storage: StorageManager<S, Read<S, VM>, Write<S, VM>>,
    scheduler: Scheduler<S, VM>,
    executor: Executor<S, VM>,
    notarization: NotarizationWorker<S, VM>,
    vm: VM,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> Runtime<S, VM> {
    pub fn process(&mut self, transactions: Vec<VM::Transaction>) -> Batch<S, VM> {
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

    pub fn from_parts(execution_workers: usize, vm: VM, storage_config: StorageConfig<S>) -> Self {
        let storage = StorageManager::new(storage_config);
        let executor = Executor::new(execution_workers, vm.clone());
        let notarization = NotarizationWorker::new(vm.clone());

        Self { scheduler: Scheduler::new(storage.clone()), executor, notarization, storage, vm }
    }
}
