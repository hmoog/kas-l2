use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use crossbeam_deque::Injector;

use crate::{
    resource_guard::ResourceGuard,
    scheduled_element::ScheduledElement,
    types::{AccessType, Element, Guards},
};

pub struct Scheduler<E: Element> {
    scheduled_elements: Vec<Arc<ScheduledElement<E>>>,
    latest_guards: HashMap<E::ResourceID, Arc<ResourceGuard<E>>>,
    injector: Arc<Injector<Arc<ScheduledElement<E>>>>,
}

impl<E: Element> Scheduler<E> {
    pub fn new() -> Self {
        Self {
            scheduled_elements: Vec::new(),
            latest_guards: HashMap::new(),
            injector: Arc::new(Injector::new()),
        }
    }

    pub fn schedule(&mut self, elements: Vec<E>) {
        for (i, element) in elements.into_iter().enumerate() {
            let guards = self.fetch_guards(i, &element);

            self.scheduled_elements
                .push(ScheduledElement::new(element, guards, &self.injector));
        }
    }

    fn fetch_guards(&mut self, index: usize, element: &E) -> Guards<E> {
        let mut prev_guards = Vec::new();
        let mut guards = Vec::new();

        let mut collect = |locks: &[E::ResourceID], access: AccessType| {
            for res in locks {
                let (prev_guard, guard) = match self.latest_guards.entry(res.clone()) {
                    Entry::Occupied(entry) if entry.get().owner_index == index => {
                        continue; // skip duplicate for the same element
                    }
                    Entry::Occupied(mut entry) => {
                        let guard = Arc::new(ResourceGuard::new(access, index));
                        let prev_guard = Some(entry.insert(guard.clone()));
                        (prev_guard, guard)
                    }
                    Entry::Vacant(entry) => {
                        let guard = Arc::new(ResourceGuard::new(access, index));
                        (None, entry.insert(guard).clone())
                    }
                };

                prev_guards.push(prev_guard);
                guards.push(guard);
            }
        };

        collect(element.write_locks(), AccessType::WriteAccess);
        collect(element.read_locks(), AccessType::ReadAccess);

        (prev_guards, guards)
    }
}
