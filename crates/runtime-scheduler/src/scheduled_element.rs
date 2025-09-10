use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use crossbeam_deque::Injector;

use crate::{
    resource_guard::ResourceGuard,
    types::{Element, Guards},
};

pub struct ScheduledElement<E: Element> {
    pub element: E,
    is_done: AtomicBool,
    pending_requests: AtomicU64,
    lock_requests: Vec<Arc<ResourceGuard<E>>>,
    injector: Arc<Injector<Arc<ScheduledElement<E>>>>,
}

impl<E: Element> ScheduledElement<E> {
    pub fn new(
        element: E,
        guards: Guards<E>,
        injector: &Arc<Injector<Arc<ScheduledElement<E>>>>,
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

    pub fn notify_ready(self: &Arc<Self>) {
        if self.pending_requests.fetch_sub(1, Ordering::AcqRel) - 1 == 0 {
            self.injector.push(self.clone())
        }
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
}
