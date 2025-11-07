use std::collections::HashMap;

use kas_l2_storage_manager::{StorageManager, Store};
use tap::Tap;

use crate::{
    AccessMetadata, Batch, BatchRef, Read, Resource, StateDiff, VecExt, Vm, Write,
    execution::runtime_tx::RuntimeTxRef, resources::resource_access::ResourceAccess,
    storage::runtime_state::RuntimeState,
};

pub struct Scheduler<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    batch_index: u64,
    storage: StorageManager<S, Read<S, VM>, Write<S, VM>>,
    resources: HashMap<VM::ResourceId, Resource<S, VM>>,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> Scheduler<S, VM> {
    pub fn new(storage: StorageManager<S, Read<S, VM>, Write<S, VM>>) -> Self {
        Self { storage, resources: HashMap::new(), batch_index: 0 }
    }

    pub fn batch_index(&self) -> u64 {
        self.batch_index
    }

    pub fn storage(&self) -> &StorageManager<S, Read<S, VM>, Write<S, VM>> {
        &self.storage
    }

    pub fn schedule(&mut self, txs: Vec<VM::Transaction>) -> Batch<S, VM> {
        self.batch_index += 1;
        Batch::new(self, txs).tap(Batch::connect)
    }

    pub(crate) fn resources(
        &mut self,
        tx: &VM::Transaction,
        runtime_tx: RuntimeTxRef<S, VM>,
        batch: &BatchRef<S, VM>,
        state_diffs: &mut Vec<StateDiff<S, VM>>,
    ) -> Vec<ResourceAccess<S, VM>> {
        tx.accessed_resources().iter().into_vec(|access| {
            self.resources.entry(access.id()).or_default().access(access, &runtime_tx, batch).tap(
                |access| {
                    if access.is_batch_head() {
                        state_diffs.push(access.state_diff());
                    }
                },
            )
        })
    }
}
