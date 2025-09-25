use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    atomic::{AtomicOptionArc, AtomicWeak},
    resources::{AccessHandle, AtomicAccessor, access::Access},
    transactions::Transaction,
};
use crate::resources::State;

pub struct AtomicAccess<T: Transaction, A: AtomicAccessor> {
    accessor: AtomicWeak<A>,
    accesses: Vec<AtomicOptionArc<Access<T, A>>>,
    pending_accesses: AtomicU64,
}

impl<T: Transaction, A: AtomicAccessor> AtomicAccess<T, A> {
    pub fn new(x: Vec<Arc<Access<T, A>>>) -> Self {
        Self {
            pending_accesses: AtomicU64::new(x.len() as u64),
            accessor: AtomicWeak::default(),
            accesses: x.into_iter().map(|y| AtomicOptionArc::new(Some(y))).collect(),
        }
    }

    pub fn init_missing_resources(self: Arc<Self>) -> Arc<Self> {
        for access in self.accesses.iter() {
            let access = access.load().unwrap();
            match access.prev_access() {
                Some(prev) => prev.publish_next_access(&access),
                None => {
                    // TODO: no previous guard -> read from underlying storage!
                    access.publish_loaded_state(Arc::new(State::new(
                        T::ResourceID::default(),
                        Vec::new(),
                        0,
                        false,
                    )))
                }
            }
        }

        self
    }

    pub fn init_accessor(self: &Arc<Self>, accessor: &Arc<A>) {
        self.accessor.store(Arc::downgrade(accessor));

        if self.pending_accesses.load(Ordering::Acquire) == 0 {
            accessor.notify();
        }
    }

    pub fn consume<F: FnOnce(&mut [AccessHandle<T>])>(&self, processor: F) {
        let mut handles: Vec<AccessHandle<T>> = self
            .accesses
            .iter()
            .map(|access| {
                let access = access.load().expect("missing access");
                let state = access.loaded_state().expect("missing state");
                state.cow_handle(access.metadata().clone())
            })
            .collect();

        processor(&mut handles);

        for (handle, access) in handles.into_iter().zip(self.accesses.iter()) {
            if let Some(access) = access.load() {
                access.publish_written_state(handle.commit());
            }
        }
    }

    pub(crate) fn notify(self: &Arc<Self>, access: &Arc<Access<T, A>>, index: usize) {
        self.accesses.get(index).unwrap().publish(access.clone());

        if self.pending_accesses.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(accessor) = self.accessor.load().upgrade() {
                accessor.notify();
            }
        }
    }
}
