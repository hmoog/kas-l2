use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use crate::{
    AccessMetadata, Resource, ResourceState, ResourcesConsumer, ResourcesProvider, Transaction,
};

pub struct ResourceManager<T: Transaction, C: ResourcesConsumer> {
    guards: HashMap<T::ResourceID, Resource<T, C>>,
}

impl<T: Transaction, C: ResourcesConsumer> ResourceManager<T, C> {
    pub fn provide(&mut self, transaction: &T) -> Arc<ResourcesProvider<T, C>> {
        let mut new_providers = Vec::new();

        let resources_provider = Arc::new_cyclic(|this| {
            for access in transaction.accessed_resources() {
                new_providers.push(match self.guards.entry(access.resource_id().clone()) {
                    Entry::Occupied(entry) if entry.get().was_last_accessed_by(this) => {
                        continue; // TODO: CHANGE TO ERROR
                    }
                    Entry::Occupied(mut entry) => entry
                        .get_mut()
                        .provide(access.clone(), (this.clone(), new_providers.len())),
                    Entry::Vacant(entry) => {
                        entry
                            .insert(Resource::new())
                            .provide(access.clone(), (this.clone(), new_providers.len()))

                        // TODO: RETRIEVE DATA FROM SOURCE AND SET READY IF POSSIBLE
                    }
                });
            }

            ResourcesProvider::new(new_providers.len())
        });

        for provider in &new_providers {
            match provider.prev() {
                Some(prev) => prev.extend(provider),
                None => {
                    // TODO: no previous guard -> read from underlying storage!
                    provider.publish_read_value(Arc::new(ResourceState::new(
                        T::ResourceID::default(),
                        Vec::new(),
                        0,
                        false,
                    )))
                }
            }
        }

        resources_provider
    }
}

impl<T: Transaction, C: ResourcesConsumer> Default for ResourceManager<T, C> {
    fn default() -> Self {
        Self {
            guards: HashMap::new(),
        }
    }
}
