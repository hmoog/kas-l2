use std::{collections::HashMap};

use kas_l2_runtime_execution_workers::ExecutionWorkers;
use kas_l2_runtime_state::StateSpace;
use kas_l2_runtime_types::{AccessMetadata, Transaction};
use kas_l2_storage_manager::{StorageConfig, StorageManager};
use kas_l2_storage_types::Store;
use tap::Tap;

use crate::{
    ExecutionConfig, Read, Resource, ResourceAccess, Rollback, RuntimeBatch, RuntimeBatchRef,
    RuntimeContext, RuntimeTxRef, StateDiff, WorkerLoop, Write, cpu_task::ManagerTask,
    vm_interface::VmInterface,
};

pub struct RuntimeManager<S: Store<StateSpace = StateSpace>, V: VmInterface> {
    vm: V,
    context: RuntimeContext,
    storage_manager: StorageManager<S, Read<S, V>, Write<S, V>>,
    resources: HashMap<V::ResourceId, Resource<S, V>>,
    worker_loop: WorkerLoop<S, V>,
    execution_workers: ExecutionWorkers<ManagerTask<S, V>, RuntimeBatch<S, V>>,
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> RuntimeManager<S, V> {
    pub fn new(execution_config: ExecutionConfig<V>, storage_config: StorageConfig<S>) -> Self {
        let (worker_count, vm) = execution_config.unpack();
        Self {
            context: RuntimeContext::new(0),
            worker_loop: WorkerLoop::new(vm.clone()),
            storage_manager: StorageManager::new(storage_config),
            resources: HashMap::new(),
            execution_workers: ExecutionWorkers::new(worker_count),
            vm,
        }
    }

    pub fn context(&self) -> &RuntimeContext {
        &self.context
    }

    pub fn storage_manager(&self) -> &StorageManager<S, Read<S, V>, Write<S, V>> {
        &self.storage_manager
    }

    /// Schedules a batch of transactions for execution.
    ///
    /// This creates a new `RuntimeBatch`, connects it to the manager, pushes it to the worker loop,
    /// and submits it to the execution workers for processing.
    pub fn schedule(&mut self, txs: Vec<V::Transaction>) -> RuntimeBatch<S, V> {
        RuntimeBatch::new(self.vm.clone(), self, txs).tap(RuntimeBatch::connect).tap(|batch| {
            self.worker_loop.push(batch.clone());
            self.execution_workers.execute(batch.clone())
        })
    }

    /// Rolls back the runtime state to `target_index` if the current state is ahead of it.
    ///
    /// This updates the longest chain to reflect the rollback and submits a rollback command to the
    /// storage manager. The call blocks until the rollback completes, after which all in-memory
    /// resource pointers are cleared, as their state may have changed.
    pub fn rollback_to(&mut self, target_index: u64) {
        // Determine the range of batches to roll back.
        let lower_bound = target_index + 1;
        let upper_bound = self.context.last_batch_index();

        // Only perform a rollback if there is state to revert.
        if upper_bound >= lower_bound {
            // Update the chain; this sets the rollback threshold and cancels in-flight batches.
            self.context.rollback(target_index);

            // Submit the rollback command and wait for its completion.
            let done_signal = Default::default();
            self.storage_manager.submit_write(Write::Rollback(Rollback::new(
                lower_bound,
                upper_bound,
                &done_signal,
            )));
            done_signal.wait_blocking();

            // Clear in-memory resource pointers, as their state may no longer be valid.
            self.resources.clear();
        }
    }

    pub fn shutdown(self) {
        self.worker_loop.shutdown();
        self.execution_workers.shutdown();
        self.storage_manager.shutdown();
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
