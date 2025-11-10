use std::collections::HashMap;

use kas_l2_runtime_types::{AccessMetadata, Transaction};
use kas_l2_runtime_execution_workers::ExecutionWorkers;
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;
use kas_l2_storage_manager::{StorageConfig, StorageManager};
use tap::Tap;

use crate::{
    ExecutionConfig, Read, Resource, ResourceAccess, RuntimeBatch, RuntimeBatchRef, RuntimeTx,
    RuntimeTxRef, StateDiff, WorkerLoop, Write, vm_interface::VmInterface,
};

pub struct RuntimeManager<S: Store<StateSpace = StateSpace>, V: VmInterface> {
    vm: V,
    batch_index: u64,
    storage: StorageManager<S, Read<S, V>, Write<S, V>>,
    resources: HashMap<V::ResourceId, Resource<S, V>>,
    worker_loop: WorkerLoop<S, V>,
    executor: ExecutionWorkers<RuntimeTx<S, V>, RuntimeBatch<S, V>>,
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> RuntimeManager<S, V> {
    pub fn new(execution_config: ExecutionConfig<V>, storage_config: StorageConfig<S>) -> Self {
        let (worker_count, vm) = execution_config.unpack();
        Self {
            worker_loop: WorkerLoop::new(vm.clone()),
            storage: StorageManager::new(storage_config),
            resources: HashMap::new(),
            batch_index: 0,
            executor: ExecutionWorkers::new(worker_count),
            vm,
        }
    }

    pub fn batch_index(&self) -> u64 {
        self.batch_index
    }

    pub fn storage(&self) -> &StorageManager<S, Read<S, V>, Write<S, V>> {
        &self.storage
    }

    pub fn schedule(&mut self, txs: Vec<V::Transaction>) -> RuntimeBatch<S, V> {
        self.batch_index += 1;
        RuntimeBatch::new(self.vm.clone(), self, txs).tap(RuntimeBatch::connect).tap(|batch| {
            self.worker_loop.push(batch.clone());
            self.executor.execute(batch.clone())
        })
    }

    pub fn shutdown(self) {
        self.worker_loop.shutdown();
        self.executor.shutdown();
        self.storage.shutdown();
    }

    pub(crate) fn resources(
        &mut self,
        tx: &V::Transaction,
        runtime_tx: RuntimeTxRef<S, V>,
        batch: &RuntimeBatchRef<S, V>,
        state_diffs: &mut Vec<StateDiff<S, V>>,
    ) -> Vec<ResourceAccess<S, V>> {
        tx.accessed_resources()
            .iter()
            .map(|access| {
                self.resources
                    .entry(access.id())
                    .or_default()
                    .access(access, &runtime_tx, batch)
                    .tap(|access| {
                        if access.is_batch_head() {
                            state_diffs.push(access.state_diff());
                        }
                    })
            })
            .collect()
    }
}
