use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use crate::{
    PendingTasks,
    guard::{Guard, Guards, Type},
    scheduled_task::ScheduledTask,
    task::Task,
};

pub struct Batch<T: Task> {
    scheduled_tasks: Vec<Arc<ScheduledTask<T>>>,
    guards: HashMap<T::ResourceID, Arc<Guard<T>>>,
    pending_tasks: Arc<PendingTasks<T>>,
}

impl<E: Task> Batch<E> {
    pub fn new(elements: Vec<E>) -> Self {
        let mut this = Self {
            scheduled_tasks: Vec::new(),
            guards: HashMap::new(),
            pending_tasks: Arc::new(PendingTasks::new(elements.len() as u64)),
        };

        for (i, element) in elements.into_iter().enumerate() {
            let guards = this.guards(i, &element);

            this.scheduled_tasks.push(ScheduledTask::new(
                element,
                guards,
                this.pending_tasks.clone(),
            ));
        }

        this
    }

    pub fn pending_tasks(&self) -> Arc<PendingTasks<E>> {
        self.pending_tasks.clone()
    }

    fn guards(&mut self, index: usize, element: &E) -> Guards<E> {
        let mut prev_guards = Vec::new();
        let mut guards = Vec::new();

        let mut collect = |locks: &[E::ResourceID], access: Type| {
            for res in locks {
                let (prev_guard, guard) = match self.guards.entry(res.clone()) {
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
