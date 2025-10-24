use std::collections::HashMap;

use kas_l2_storage::{Storage, Store};
use tap::Tap;

use crate::{
    AccessMetadata, Batch, BatchRef, Resource, StateDiff, Transaction,
    execution::runtime_tx::RuntimeTxRef,
    resources::resource_access::ResourceAccess,
    storage::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
    utils::vec_ext::VecExt,
};

pub struct Scheduler<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    storage: Storage<S, Read<S, T>, Write<S, T>>,
    resources: HashMap<T::ResourceId, Resource<S, T>>,
    batch_index: u64,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> Scheduler<S, T> {
    pub fn new(storage: Storage<S, Read<S, T>, Write<S, T>>) -> Self {
        Self {
            storage,
            resources: HashMap::new(),
            batch_index: 0,
        }
    }

    pub fn storage(&self) -> &Storage<S, Read<S, T>, Write<S, T>> {
        &self.storage
    }

    pub fn schedule(&mut self, txs: Vec<T>) -> Batch<S, T> {
        self.batch_index += 1;
        Batch::new(self, self.batch_index, txs)
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
