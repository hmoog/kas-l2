use std::{collections::HashMap, sync::Arc};

use borsh::BorshDeserialize;

use crate::{
    AccessMetadata, BatchApiRef, Resource, ResourceAccess, RuntimeTxRef, State, StateDiff, Storage,
    Transaction, VecExt,
};

pub struct ResourceProvider<T: Transaction, K: Storage<T::ResourceId>> {
    resources: HashMap<T::ResourceId, Resource<T>>,
    permanent_storage: K,
}

impl<T: Transaction, K: Storage<T::ResourceId>> ResourceProvider<T, K> {
    pub(crate) fn new(permanent_storage: K) -> Self {
        Self {
            resources: HashMap::new(),
            permanent_storage,
        }
    }

    pub(crate) fn provide(
        &mut self,
        tx: &T,
        runtime_tx: RuntimeTxRef<T>,
        batch: &BatchApiRef<T>,
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

    pub(crate) fn load_from_storage(&self, resource: &ResourceAccess<T>) {
        resource.set_read_state(Arc::new(match self.permanent_storage.get(&resource.id()) {
            Ok(result) => match result {
                None => State::default(),
                Some(bytes) => State::try_from_slice(&bytes).expect("failed to deserialize"),
            },
            Err(err) => panic!("failed to load resource from storage: {}", err),
        }))
    }

    fn resource(&mut self, resource_id: T::ResourceId) -> &mut Resource<T> {
        self.resources.entry(resource_id).or_default()
    }
}
