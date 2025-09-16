use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use crate::{
    PendingTasks,
    guard::{Guard, Guards},
    task::Task,
};

pub struct ScheduledTask<E: Task> {
    element: E,
    guards: Vec<Arc<Guard<E>>>,
    pending_guards: AtomicU64,
    is_done: AtomicBool,
    pending_tasks: Arc<PendingTasks<E>>,
}

impl<E: Task> ScheduledTask<E> {
    pub(crate) fn new(
        element: E,
        guards: Guards<E>,
        pending_tasks: Arc<PendingTasks<E>>,
    ) -> Arc<Self> {
        let this = Arc::new_cyclic(|weak| {
            guards
                .1
                .iter()
                .for_each(|request| request.owner.store(weak.clone()));

            Self {
                element,
                pending_guards: AtomicU64::new(guards.1.len() as u64),
                is_done: AtomicBool::new(false),
                guards: guards.1,
                pending_tasks,
            }
        });

        // connect new requests to old ones to be notified when they are done
        for (prev_guard, guard) in guards.0.into_iter().zip(this.guards.iter()) {
            match prev_guard {
                None => guard.ready(),
                Some(prev_guard) => prev_guard.extend(guard),
            }
        }

        this
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

    pub(crate) fn notify_ready(self: &Arc<Self>) {
        if self.pending_guards.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.pending_tasks.ready.push(self.clone())
        }
    }
}
