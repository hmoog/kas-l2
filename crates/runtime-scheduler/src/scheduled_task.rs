use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use crossbeam_deque::Injector;

use crate::{
    guard::{Guard, Guards},
    task::Task,
};

pub struct ScheduledTask<E: Task> {
    element: E,
    is_done: AtomicBool,
    pending_requests: AtomicU64,
    lock_requests: Vec<Arc<Guard<E>>>,
    injector: Arc<Injector<Arc<ScheduledTask<E>>>>,
}

impl<E: Task> ScheduledTask<E> {
    pub fn new(
        element: E,
        guards: Guards<E>,
        injector: &Arc<Injector<Arc<ScheduledTask<E>>>>,
    ) -> Arc<Self> {
        let this = Arc::new(Self {
            element,
            pending_requests: AtomicU64::new(guards.1.len() as u64),
            is_done: AtomicBool::new(false),
            lock_requests: guards.1,
            injector: injector.clone(),
        });

        // wire up ownership first (so notifications work correctly)
        for request in this.lock_requests.iter() {
            request.owner.store(Arc::downgrade(&this));
        }

        // connect new requests to old ones to be notified when they are done
        for (prev_guard, guard) in guards.0.into_iter().zip(this.lock_requests.iter()) {
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
            self.lock_requests.iter().for_each(|x| x.done());
        }
    }

    pub(crate) fn notify_ready(self: &Arc<Self>) {
        if self.pending_requests.fetch_sub(1, Ordering::AcqRel) - 1 == 0 {
            self.injector.push(self.clone())
        }
    }
}
