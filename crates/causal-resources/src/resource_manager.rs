use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use kas_l2_core::{AccessMetadata, ResourceID, Transaction};

use crate::{ResourcesConsumer, resource::Resource, resources_provider::ResourcesProvider};

pub struct ResourceManager<R: ResourceID, C: ResourcesConsumer> {
    guards: HashMap<R, Resource<ResourcesProvider<C>>>,
}

impl<R: ResourceID, C: ResourcesConsumer> ResourceManager<R, C> {
    pub fn provide<T: Transaction<ResourceID = R>>(
        &mut self,
        transaction: &T,
    ) -> Arc<ResourcesProvider<C>> {
        let mut new_resources = Vec::new();

        let resources = Arc::new_cyclic(|guards| {
            for access in transaction.accessed_resources() {
                new_resources.push(match self.guards.entry(access.resource_id().clone()) {
                    Entry::Occupied(entry) if entry.get().was_last_accessed_by(guards) => {
                        continue; // TODO: CHANGE TO ERROR
                    }
                    Entry::Occupied(mut entry) => entry
                        .get_mut()
                        .provide((guards.clone(), new_resources.len()), access.access_type()),
                    Entry::Vacant(entry) => {
                        entry
                            .insert(Resource::new())
                            .provide((guards.clone(), new_resources.len()), access.access_type())

                        // TODO: RETRIEVE DATA FROM SOURCE AND SET READY IF POSSIBLE
                    }
                });
            }

            ResourcesProvider::new(new_resources.len())
        });

        for guard in &new_resources {
            match guard.prev.load() {
                Some(prev) => prev.extend(guard),
                None => {
                    // TODO: no previous guard -> read from underlying storage!
                    guard.ready()
                }
            }
        }

        resources
    }
}

impl<R: ResourceID, C: ResourcesConsumer> Default for ResourceManager<R, C> {
    fn default() -> Self {
        Self {
            guards: HashMap::new(),
        }
    }
}
