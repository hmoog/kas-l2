use std::collections::HashMap;

use kas_l2_storage_manager::{StorageManager, Store};
use tap::Tap;

use crate::{
    AccessMetadata, Batch, BatchRef, Read, Resource, StateDiff, Transaction, VecExt, Write,
    execution::runtime_tx::RuntimeTxRef, resources::resource_access::ResourceAccess,
    storage::runtime_state::RuntimeState,
};

pub struct Scheduler<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    batch_index: u64,
    storage: StorageManager<S, Read<S, T>, Write<S, T>>,
    resources: HashMap<T::ResourceId, Resource<S, T>>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> Scheduler<S, T> {
    pub fn new(storage: StorageManager<S, Read<S, T>, Write<S, T>>) -> Self {
        Self {
            storage,
            resources: HashMap::new(),
            batch_index: 0,
        }
    }

    pub fn batch_index(&self) -> u64 {
        self.batch_index
    }

    pub fn storage(&self) -> &StorageManager<S, Read<S, T>, Write<S, T>> {
        &self.storage
    }

    pub fn schedule(&mut self, txs: Vec<T>) -> Batch<S, T> {
        self.batch_index += 1;
        Batch::new(self, txs).tap(Batch::connect)
    }

    pub(crate) fn resources(
        &mut self,
        tx: &T,
        runtime_tx: RuntimeTxRef<S, T>,
        batch: &BatchRef<S, T>,
        state_diffs: &mut Vec<StateDiff<S, T>>,
    ) -> Vec<ResourceAccess<S, T>> {
        tx.accessed_resources().iter().into_vec(|access| {
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
    }
}
