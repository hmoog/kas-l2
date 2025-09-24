use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use crate::{
    resources::{AtomicAccess, AtomicAccessor, Resource, State, access_metadata::AccessMetadata},
    transactions::Transaction,
};

pub struct ResourceManager<T: Transaction, C: AtomicAccessor> {
    resources: HashMap<T::ResourceID, Resource<T, C>>,
}

impl<T: Transaction, C: AtomicAccessor> ResourceManager<T, C> {
    pub fn access(&mut self, transaction: &T) -> Arc<AtomicAccess<T, C>> {
        let mut accesses = Vec::new();

        let atomic_access = Arc::new_cyclic(|this| {
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

            AtomicAccess::new(accesses.len())
        });

        for provider in &accesses {
            match provider.prev_access() {
                Some(prev) => prev.extend(provider),
                None => {
                    // TODO: no previous guard -> read from underlying storage!
                    provider.load(Arc::new(State::new(
                        T::ResourceID::default(),
                        Vec::new(),
                        0,
                        false,
                    )))
                }
            }
        }

        atomic_access
    }
}

impl<T: Transaction, C: AtomicAccessor> Default for ResourceManager<T, C> {
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}
