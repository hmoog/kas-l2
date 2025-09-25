use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use crate::{
    resources::{AtomicAccess, AtomicAccessor, Resource, AccessMetadata},
    transactions::Transaction,
};
use crate::resources::State;

pub struct ResourceManager<T: Transaction, C: AtomicAccessor> {
    resources: HashMap<T::ResourceID, Resource<T, C>>,
}

impl<T: Transaction, C: AtomicAccessor> ResourceManager<T, C> {
    pub fn access(&mut self, transaction: &T) -> Arc<AtomicAccess<T, C>> {
        Arc::new_cyclic(|this| {
            let mut accesses = Vec::new();
            for access in transaction.accessed_resources() {
                accesses.push(match self.resources.entry(access.resource_id().clone()) {
                    Entry::Occupied(entry) if entry.get().last_atomic_access_by(this) => {
                        continue; // TODO: CHANGE TO ERROR
                    }
                    Entry::Occupied(mut entry) => entry
                        .get_mut()
                        .access(access.clone(), (this.clone(), accesses.len())),
                    Entry::Vacant(entry) => {
                        entry
                            .insert(Resource::default())
                            .access(access.clone(), (this.clone(), accesses.len()))
                    }
                });
            }

            AtomicAccess::new(accesses)
        }).provide_resources(|access| {
            // TODO: no previous guard -> read from underlying storage!
            access.publish_loaded_state(Arc::new(State::new(
                T::ResourceID::default(),
                Vec::new(),
                0,
                false,
            )))
        })
    }
}

impl<T: Transaction, C: AtomicAccessor> Default for ResourceManager<T, C> {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}
