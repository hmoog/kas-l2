use std::{collections::HashMap, sync::Arc};

use borsh::BorshDeserialize;

use crate::{
    resources::{AccessMetadata, AtomicAccess, AtomicAccessor, Resource, State, access::Access},
    storage::KvStore,
    transactions::Transaction,
};

pub struct ResourceManager<T: Transaction, C: AtomicAccessor, K: KvStore<T::ResourceID>> {
    cached_resources: HashMap<T::ResourceID, Resource<T, C>>,
    permanent_storage: K,
}

impl<T: Transaction, C: AtomicAccessor, K: KvStore<T::ResourceID>> ResourceManager<T, C, K> {
    pub fn new(permanent_storage: K) -> Self {
        Self {
            cached_resources: HashMap::new(),
            permanent_storage,
        }
    }

    pub fn access(&mut self, transaction: &T) -> Arc<AtomicAccess<T, C>> {
        Arc::new_cyclic(|this| {
            let mut accessed_resources = Vec::new();
            for access_meta in transaction.accessed_resources() {
                let resource = self
                    .cached_resources
                    .entry(access_meta.resource_id())
                    .or_default();

                if resource.last_accessed_by(this) {
                    panic!("duplicate access to resource")
                }

                accessed_resources.push(resource.access(
                    access_meta.clone(),
                    (this.clone(), accessed_resources.len()),
                ));
            }

            AtomicAccess::new(accessed_resources)
        })
        .load_missing(|access| self.load_from_storage(access))
    }

    pub fn load_from_storage(&self, access: &Arc<Access<T, C>>) {
        access.publish_loaded_state(Arc::new(
            match self.permanent_storage.get(&access.resource_id()) {
                Ok(result) => match result {
                    None => State::default(),
                    Some(bytes) => State::try_from_slice(&bytes).expect("failed to deserialize"),
                },
                Err(err) => panic!("failed to load resource from storage: {}", err),
            },
        ))
    }
}
