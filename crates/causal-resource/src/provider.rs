use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Weak,
};

use crate::{
    access_type::AccessType, consumer::Consumer, resource::Resource, resource_id::ResourceID,
};

pub struct Provider<R: ResourceID, N: Consumer> {
    guards: HashMap<R, Resource<N>>,
}

impl<R: ResourceID, N: Consumer> Provider<R, N> {
    pub fn new() -> Self {
        Self {
            guards: HashMap::new(),
        }
    }

    fn request_access(&mut self, notifier: Weak<N>, writes: &[R], reads: &[R]) -> Guards<E> {
        let mut guards = Vec::new();
        let mut prev_guards = Vec::new();

        let mut collect = move |locks: &[R], access: AccessType| {
            for res in locks {
                let (prev_guard, guard) = match self.guards.entry(res.clone()) {
                    Entry::Occupied(entry) if entry.get().was_last_accessed_by(&notifier) => {
                        continue; // skip duplicate for the same element
                    }
                    Entry::Occupied(mut entry) => entry.get_mut().access(notifier.clone(), access),
                    Entry::Vacant(entry) => {
                        entry
                            .insert(Resource::new())
                            .access(notifier.clone(), access)
                        // TODO: RETRIEVE DATA FROM SOURCE AND SET READY IF POSSIBLE
                    }
                };

                prev_guards.push(prev_guard);
                guards.push(guard);
            }
        };

        collect(writes, AccessType::Write);
        collect(reads, AccessType::Read);

        (guards, prev_guards)
    }
}
