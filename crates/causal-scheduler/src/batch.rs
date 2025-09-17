use std::sync::Arc;

use kas_l2_causal_resource::Provider;

use crate::{PendingTasks, scheduled_task::ScheduledTask, task::Task};

pub struct Batch<T: Task> {
    scheduled_tasks: Vec<Arc<ScheduledTask<T>>>,
    pending_tasks: Arc<PendingTasks<T>>,
}

impl<E: Task> Batch<E> {
    pub fn new(elements: Vec<E>, provider: &mut Provider<E::ResourceID, ScheduledTask<E>>) -> Self {
        let mut this = Self {
            scheduled_tasks: Vec::new(),
            pending_tasks: Arc::new(PendingTasks::new(elements.len() as u64)),
        };

        for element in elements.into_iter() {
            let stash = std::cell::RefCell::new(None);
            this.scheduled_tasks.push(Arc::new_cyclic(|weak| {
                let setup =
                    provider.access(weak.clone(), element.write_locks(), element.read_locks());
                let guards = setup.guards.clone();
                *stash.borrow_mut() = Some(setup);

                ScheduledTask::new(element, guards, this.pending_tasks.clone())
            }));
            drop(stash.into_inner());
        }

        this
    }

    pub fn pending_tasks(&self) -> Arc<PendingTasks<E>> {
        self.pending_tasks.clone()
    }
}
