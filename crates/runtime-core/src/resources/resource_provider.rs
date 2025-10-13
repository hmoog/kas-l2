use std::collections::HashMap;

use crate::{
    AccessMetadata, BatchRef, Resource, ResourceAccess, RuntimeTxRef, StateDiff, Transaction,
    VecExt,
};

pub struct ResourceProvider<T: Transaction> {
    resources: HashMap<T::ResourceId, Resource<T>>,
}

impl<T: Transaction> ResourceProvider<T> {
    pub(crate) fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub(crate) fn provide(
        &mut self,
        tx: &T,
        runtime_tx: RuntimeTxRef<T>,
        batch: &BatchRef<T>,
        state_diffs: &mut Vec<StateDiff<T>>,
    ) -> Vec<ResourceAccess<T>> {
        tx.accessed_resources().iter().into_vec(|access| {
            let resource = self.resource(access.id());
            let (access, new_state_diff) = resource.access(access, &runtime_tx, batch);
            if let Some(new_state_diff) = new_state_diff {
                state_diffs.push(new_state_diff);
            }
            access
        })
    }

    fn resource(&mut self, resource_id: T::ResourceId) -> &mut Resource<T> {
        self.resources.entry(resource_id).or_default()
    }
}
