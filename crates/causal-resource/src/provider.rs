use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Weak,
};

use crate::{
    access_type::AccessType, guard_consumer::GuardConsumer, guards_setup::GuardsSetup, resource::Resource,
    resource_id::ResourceID,
};

pub struct Provider<R: ResourceID, N: GuardConsumer<GuardID=usize>> {
    guards: HashMap<R, Resource<N>>,
}

impl<R: ResourceID, N: GuardConsumer<GuardID=usize>> Provider<R, N> {
    pub fn new() -> Self {
        Self {
            guards: HashMap::new(),
        }
    }

    pub fn access(&mut self, notifier: Weak<N>, writes: &[R], reads: &[R]) -> GuardsSetup<N> {
        let mut new_guards = Vec::new();
        let mut prev_guards = Vec::new();

        let mut collect = |locks: &[R], access: AccessType| {
            for res in locks {
                let (guard, prev_guard) = match self.guards.entry(res.clone()) {
                    Entry::Occupied(entry) if entry.get().was_last_accessed_by(&notifier) => {
                        continue; // skip duplicate for the same element
                    }
                    Entry::Occupied(mut entry) => entry.get_mut().access(notifier.clone(), 0, access),
                    Entry::Vacant(entry) => {
                        entry
                            .insert(Resource::new())
                            .access(notifier.clone(), 0, access)
                        // TODO: RETRIEVE DATA FROM SOURCE AND SET READY IF POSSIBLE
                    }
                };

                new_guards.push(guard);
                prev_guards.push(prev_guard);
            }
        };

        collect(writes, AccessType::Write);
        collect(reads, AccessType::Read);

        GuardsSetup::new(new_guards, prev_guards)
    }
}
