use std::{collections::HashMap, sync::Arc};

use borsh::BorshDeserialize;

use crate::{
    resources::{
        AccessMetadata, Consumer, Resources, State, resource::Resource,
        resource_manager::ResourceManager,
    },
    storage::KvStore,
    transactions::Transaction,
};

pub struct ResourceProvider<T: Transaction, C: Consumer, K: KvStore<T::ResourceID>> {
    cached_resources: HashMap<T::ResourceID, ResourceManager<T, C>>,
    permanent_storage: K,
}

impl<T: Transaction, C: Consumer, K: KvStore<T::ResourceID>> ResourceProvider<T, C, K> {
    pub fn new(permanent_storage: K) -> Self {
        Self {
            cached_resources: HashMap::new(),
            permanent_storage,
        }
    }

    pub fn provide_resources(&mut self, transaction: &T) -> Arc<Resources<T, C>> {
        Arc::new_cyclic(|weak_resources| {
            let mut resources = Vec::new();
            for access in transaction.accessed_resources() {
                let manager = self.resource_manager(access.resource_id());
                if manager.is_duplicate_access(weak_resources) {
                    panic!("duplicate access to resource")
                }

                resources.push(manager.provide_resource(access.clone(), weak_resources.clone()));
            }

            Resources::new(resources)
        })
        .init_resources(|resource| self.load_from_storage(resource))
    }

    fn resource_manager(&mut self, resource_id: T::ResourceID) -> &mut ResourceManager<T, C> {
        self.cached_resources.entry(resource_id).or_default()
    }

    fn load_from_storage(&self, access: Arc<Resource<T, C>>) {
        let resource_id = access.resource_id();

        access.set_read_state(Arc::new(match self.permanent_storage.get(&resource_id) {
            Ok(result) => match result {
                None => State::default(),
                Some(bytes) => State::try_from_slice(&bytes).expect("failed to deserialize"),
            },
            Err(err) => panic!("failed to load resource from storage: {}", err),
        }))
    }
}
