use std::collections::HashMap;

use kas_l2_storage::{Storage, Store};

use crate::{
    AccessMetadata, BatchRef, Resource, ResourceAccess, RuntimeState, RuntimeTxRef, StateDiff,
    Transaction, VecExt,
    io::{read_cmd::Read, write_cmd::Write},
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
        storage: Storage<S, Read<S, T>, Write<S, T>>,
        tx: &T,
        runtime_tx: RuntimeTxRef<S, T>,
        batch: &BatchRef<S, T>,
        state_diffs: &mut Vec<StateDiff<S, T>>,
    ) -> Vec<ResourceAccess<S, T>> {
        tx.accessed_resources().iter().into_vec(|access| {
            let resource = self.resource(access.id());
            let (access, new_state_diff) =
                resource.access(storage.clone(), access, &runtime_tx, batch);
            if let Some(new_state_diff) = new_state_diff {
                state_diffs.push(new_state_diff);
            }
            access
        })
    }

    fn resource(&mut self, resource_id: T::ResourceId) -> &mut Resource<S, T> {
        self.resources.entry(resource_id).or_default()
    }
}
