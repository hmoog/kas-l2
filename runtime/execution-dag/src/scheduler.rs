use std::collections::HashMap;

use kas_l2_runtime_interface::{AccessMetadata, Transaction};
use kas_l2_runtime_state_space::StateSpace;
use kas_l2_runtime_storage_manager::StorageManager;
use kas_l2_storage_interface::Store;
use tap::Tap;

use crate::{
    Batch, BatchRef, Read, Resource, ResourceAccess, RuntimeTxRef, StateDiff, Write, vm::VM,
};

pub struct Scheduler<S: Store<StateSpace = StateSpace>, V: VM> {
    vm: V,
    batch_index: u64,
    storage: StorageManager<S, Read<S, V>, Write<S, V>>,
    resources: HashMap<V::ResourceId, Resource<S, V>>,
}

impl<S: Store<StateSpace = StateSpace>, V: VM> Scheduler<S, V> {
    pub fn new(vm: V, storage: StorageManager<S, Read<S, V>, Write<S, V>>) -> Self {
        Self { vm, storage, resources: HashMap::new(), batch_index: 0 }
    }

    pub fn batch_index(&self) -> u64 {
        self.batch_index
    }

    pub fn storage(&self) -> &StorageManager<S, Read<S, V>, Write<S, V>> {
        &self.storage
    }

    pub fn schedule(&mut self, txs: Vec<V::Transaction>) -> Batch<S, V> {
        self.batch_index += 1;
        Batch::new(self.vm.clone(), self, txs).tap(Batch::connect)
    }

    pub(crate) fn resources(
        &mut self,
        tx: &V::Transaction,
        runtime_tx: RuntimeTxRef<S, V>,
        batch: &BatchRef<S, V>,
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
