use std::collections::HashMap;

use kas_l2_storage::Store;
use tap::Tap;

use crate::{
    AccessMetadata, BatchRef, Resource, ResourceAccess, RuntimeState, RuntimeTxRef, StateDiff,
    Transaction, VecExt,
};

pub struct ResourceProvider<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    resources: HashMap<T::ResourceId, Resource<S, T>>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> ResourceProvider<S, T> {
    pub(crate) fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub(crate) fn provide(
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
