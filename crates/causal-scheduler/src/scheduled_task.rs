use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use kas_l2_causal_resource::{GuardConsumer, Guard};

use crate::{PendingTasks, task::Task};

pub struct ScheduledTask<E: Task> {
    element: E,
    guards: Arc<Vec<Arc<Guard<ScheduledTask<E>>>>>,
    pending_guards: AtomicU64,
    is_done: AtomicBool,
    pending_tasks: Arc<PendingTasks<E>>,
}

impl<E: Task> ScheduledTask<E> {
    pub(crate) fn new(
        task: E,
        guards: Arc<Vec<Arc<Guard<ScheduledTask<E>>>>>,
        pending_tasks: Arc<PendingTasks<E>>,
    ) -> Self {
        Self {
            element: task,
            pending_guards: AtomicU64::new(guards.len() as u64),
            is_done: AtomicBool::new(false),
            guards,
            pending_tasks,
        }
    }

    pub fn element(&self) -> &E {
        &self.element
    }

    pub fn done(&self) {
        if self
            .is_done
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.guards.iter().for_each(|guard| guard.done());

            if self.pending_tasks.count.fetch_sub(1, Ordering::AcqRel) == 1 {
                self.pending_tasks.done.open()
            }
        }
    }
}

impl<T: Task> GuardConsumer for ScheduledTask<T> {
    type GuardID = usize;
    fn notify(self: &Arc<Self>, _key: &Self::GuardID) {
        if self.pending_guards.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.pending_tasks.ready.push(self.clone())
        }
    }
}
