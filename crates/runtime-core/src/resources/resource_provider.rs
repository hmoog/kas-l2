use std::{collections::HashMap, sync::Arc};

use borsh::BorshDeserialize;

use crate::{
    AccessMetadata, Resource, ResourceAccess, RuntimeTxRef, State, Storage, Transaction, VecExt,
};

pub struct ResourceProvider<T: Transaction, K: Storage<T::ResourceId>> {
    resources: HashMap<T::ResourceId, Resource<T>>,
    permanent_storage: K,
}

impl<T: Transaction, K: Storage<T::ResourceId>> ResourceProvider<T, K> {
    pub fn new(permanent_storage: K) -> Self {
        Self {
            resources: HashMap::new(),
            permanent_storage,
        }
    }

    pub(crate) fn provide(
        &mut self,
        transaction: &T,
        tx_ref: RuntimeTxRef<T>,
    ) -> Vec<ResourceAccess<T>> {
        transaction.accessed_resources().iter().into_vec(|access| {
            let resource = self.resource(access.id());
            if resource
                .last_access()
                .is_some_and(|a| *a.tx_ref() == tx_ref)
            {
                panic!("duplicate access to resource");
            }
            resource.access(access.clone(), tx_ref.clone())
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
