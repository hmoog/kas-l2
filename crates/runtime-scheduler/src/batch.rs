use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};
use std::sync::atomic::AtomicU64;
use crossbeam_deque::Injector;

use crate::{
    guard::{Guard, Guards, Type},
    scheduled_task::ScheduledTask,
    task::Task,
};

pub struct Batch<T: Task> {
    scheduled_elements: Vec<Arc<ScheduledTask<T>>>,
    latest_guards: HashMap<T::ResourceID, Arc<Guard<T>>>,
    injector: Arc<Injector<Arc<ScheduledTask<T>>>>,
    pending_tasks: Arc<AtomicU64>,
}

impl<E: Task> Batch<E> {
    pub fn new(elements: Vec<E>) -> Self {
        let mut this = Self {
            scheduled_elements: Vec::new(),
            latest_guards: HashMap::new(),
            injector: Arc::new(Injector::new()),
            pending_tasks: Arc::new(AtomicU64::new(elements.len() as u64)),
        };

        for (i, element) in elements.into_iter().enumerate() {
            let guards = this.guards(i, &element);

            this.scheduled_elements
                .push(ScheduledTask::new(element, guards, &this.injector));
        }

        this
    }

    pub fn injector(&self) -> Arc<Injector<Arc<ScheduledTask<E>>>> {
        self.injector.clone()
    }

    fn guards(&mut self, index: usize, element: &E) -> Guards<E> {
        let mut prev_guards = Vec::new();
        let mut guards = Vec::new();

        let mut collect = |locks: &[E::ResourceID], access: Type| {
            for res in locks {
                let (prev_guard, guard) = match self.latest_guards.entry(res.clone()) {
                    Entry::Occupied(entry) if entry.get().owner_index == index => {
                        continue; // skip duplicate for the same element
                    }
                    Entry::Occupied(mut entry) => {
                        let guard = Arc::new(Guard::new(access, index));
                        let prev_guard = Some(entry.insert(guard.clone()));
                        (prev_guard, guard)
                    }
                    Entry::Vacant(entry) => {
                        let guard = Arc::new(Guard::new(access, index));
                        (None, entry.insert(guard).clone())
                    }
                };

                prev_guards.push(prev_guard);
                guards.push(guard);
            }
        };

        collect(element.write_locks(), Type::WriteGuard);
        collect(element.read_locks(), Type::ReadGuard);

        (prev_guards, guards)
    }
}
