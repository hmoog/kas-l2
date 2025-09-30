use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use borsh::BorshDeserialize;

use crate::{
    AccessMetadata, BatchAPI, Storage, Transaction,
    resources::{resource::Resource, resource_access::ResourceAccess, state::State},
    scheduling::scheduled_transaction::ScheduledTransaction,
};

pub struct ResourceProvider<T: Transaction, K: Storage<T::ResourceID>> {
    resources: HashMap<T::ResourceID, Resource<T>>,
    permanent_storage: K,
}

impl<T: Transaction, K: Storage<T::ResourceID>> ResourceProvider<T, K> {
    pub fn new(permanent_storage: K) -> Self {
        Self {
            resources: HashMap::new(),
            permanent_storage,
        }
    }

    pub(crate) fn provide_resources(
        &mut self,
        transaction: &T,
        scheduled_transaction: &Weak<ScheduledTransaction<T>>,
        batch: Arc<BatchAPI<T>>,
    ) -> Vec<Arc<ResourceAccess<T>>> {
        let mut accessed_resources = Vec::new();
        for access in transaction.accessed_resources() {
            let resource = self.resource(access.resource_id());
            if resource.was_accessed_by(scheduled_transaction) {
                panic!("duplicate access to resource")
            }

            accessed_resources.push(resource.access(
                access.clone(),
                batch.clone(),
                scheduled_transaction.clone(),
            ));
        }
        accessed_resources
    }

    fn resource(&mut self, resource_id: T::ResourceID) -> &mut Resource<T> {
        self.resources.entry(resource_id).or_default()
    }

    pub(crate) fn load_from_storage(&self, access: Arc<ResourceAccess<T>>) {
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
