use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use borsh::BorshDeserialize;

use crate::{
    AccessMetadata, ScheduledTransaction, State, resource::Resource,
    resource_manager::ResourceManager, storage::Storage, transaction::Transaction,
};

pub struct ResourceProvider<T: Transaction, K: Storage<T::ResourceID>> {
    managers: HashMap<T::ResourceID, ResourceManager<T>>,
    permanent_storage: K,
}

impl<T: Transaction, K: Storage<T::ResourceID>> ResourceProvider<T, K> {
    pub fn new(permanent_storage: K) -> Self {
        Self {
            managers: HashMap::new(),
            permanent_storage,
        }
    }

    pub(crate) fn provide_resources(
        &mut self,
        transaction: &T,
        scheduled_transaction: &Weak<ScheduledTransaction<T>>,
    ) -> Vec<Arc<Resource<T>>> {
        let mut resources = Vec::new();
        for access in transaction.accessed_resources() {
            let manager = self.manager(access.resource_id());
            if manager.has_duplicate_access(scheduled_transaction) {
                panic!("duplicate access to resource")
            }

            resources.push(manager.provide_resource(access.clone(), scheduled_transaction.clone()));
        }
        resources
    }

    fn manager(&mut self, resource_id: T::ResourceID) -> &mut ResourceManager<T> {
        self.managers.entry(resource_id).or_default()
    }

    pub(crate) fn load_from_storage(&self, access: Arc<Resource<T>>) {
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
