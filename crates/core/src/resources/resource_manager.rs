use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use crate::{
    resources::{AtomicAccess, AtomicAccessor, Resource, AccessMetadata},
    transactions::Transaction,
};

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

                        // TODO: RETRIEVE DATA FROM SOURCE AND SET READY IF POSSIBLE
                    }
                });
            }

            AtomicAccess::new(accesses)
        }).init_missing_resources()
    }
}

impl<T: Transaction, C: AtomicAccessor> Default for ResourceManager<T, C> {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}
