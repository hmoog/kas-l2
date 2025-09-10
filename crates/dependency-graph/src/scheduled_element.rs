use crate::element::Element;
use crate::lock_request::LockRequest;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct ScheduledElement<E: Element> {
    pub element: E,
    pub index: usize,
    pub is_done: AtomicBool,
    pub lock_requests: Vec<Arc<LockRequest<E>>>,
}

impl<E: Element> ScheduledElement<E> {
    pub fn new(
        element: E,
        index: usize,
        lock_requests: Vec<Arc<LockRequest<E>>>,
        old_requests: Vec<Option<Arc<LockRequest<E>>>>,
    ) -> Arc<Self> {
        let this = Arc::new(Self {
            element,
            index,
            is_done: AtomicBool::new(false),
            lock_requests,
        });

        // wire up ownership first (so notifications work correctly)
        for request in this.lock_requests.iter() {
            request.owner.store(Arc::downgrade(&this));
        }

        // connect new requests to old ones to be notified when they are done
        for (old_request, new_request) in old_requests.into_iter().zip(this.lock_requests.iter()) {
            match old_request {
                None => new_request.acquire(),
                Some(old_request) => old_request.notify(new_request),
            }
        }

        this
    }

    pub fn acquire_lock(&self) {
        // Placeholder for any logic needed when a lock is acquired
    }

    pub fn done(&self) {
        if self
            .is_done
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .is_ok()
        {
            self.lock_requests.iter().for_each(|x| x.release());
        }
    }
}
