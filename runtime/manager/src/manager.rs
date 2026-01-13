use std::collections::HashMap;

use kas_l2_runtime_execution_workers::ExecutionWorkers;
use kas_l2_runtime_state::StateSpace;
use kas_l2_runtime_types::{AccessMetadata, Transaction};
use kas_l2_storage_manager::{StorageConfig, StorageManager};
use kas_l2_storage_types::Store;
use tap::Tap;

use crate::{
    Chain, ExecutionConfig, Read, Resource, ResourceAccess, Rollback, RuntimeBatch,
    RuntimeBatchRef, RuntimeTxRef, StateDiff, WorkerLoop, Write, cpu_task::ManagerTask,
    vm_interface::VmInterface,
};

pub struct RuntimeManager<S: Store<StateSpace = StateSpace>, V: VmInterface> {
    vm: V,
    longest_chain: Chain,
    storage_manager: StorageManager<S, Read<S, V>, Write<S, V>>,
    resources: HashMap<V::ResourceId, Resource<S, V>>,
    worker_loop: WorkerLoop<S, V>,
    execution_workers: ExecutionWorkers<ManagerTask<S, V>, RuntimeBatch<S, V>>,
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> RuntimeManager<S, V> {
    pub fn new(execution_config: ExecutionConfig<V>, storage_config: StorageConfig<S>) -> Self {
        let (worker_count, vm) = execution_config.unpack();
        Self {
            longest_chain: Chain::new(0),
            worker_loop: WorkerLoop::new(vm.clone()),
            storage_manager: StorageManager::new(storage_config),
            resources: HashMap::new(),
            execution_workers: ExecutionWorkers::new(worker_count),
            vm,
        }
    }

    pub fn longest_chain(&self) -> &Chain {
        &self.longest_chain
    }

    pub fn storage_manager(&self) -> &StorageManager<S, Read<S, V>, Write<S, V>> {
        &self.storage_manager
    }

    pub fn schedule(&mut self, txs: Vec<V::Transaction>) -> RuntimeBatch<S, V> {
        RuntimeBatch::new(self.vm.clone(), self, txs).tap(RuntimeBatch::connect).tap(|batch| {
            self.worker_loop.push(batch.clone());
            self.execution_workers.execute(batch.clone())
        })
    }

    /// Rolls back the chain to the given index, reverting all state changes
    /// from batches after the target index.
    ///
    /// This method:
    /// 1. Updates the chain's rollback threshold to cancel in-flight batches
    /// 2. Submits a rollback write command to revert persisted state changes
    /// 3. Waits for the rollback to complete
    /// 4. Clears in-memory resource pointers
    ///
    /// After rollback, the chain will continue from the given index.
    pub fn rollback(&mut self, target_index: u64) {
        // Get the current last batch index before updating the chain
        let last_batch_index = self.longest_chain.last_batch_index();

        // Only submit rollback if there are batches to roll back
        if last_batch_index > target_index {
            // Update the chain - this sets the rollback threshold which cancels in-flight batches
            self.longest_chain = self.longest_chain.rollback(target_index);

            // Create rollback command
            let rollback = Rollback::new(last_batch_index, target_index);

            // Retrieve latch to be able to wait for rollback completion
            let done_latch = rollback.done_latch();

            // Submit the rollback command
            self.storage_manager.submit_write(Write::Rollback(rollback));

            // Wait for rollback to complete
            done_latch.wait_blocking();

            // Clear all in-memory resource pointers since their state may have changed
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
