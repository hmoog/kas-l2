use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    atomic::{AtomicOptionArc, AtomicWeak},
    resources::{AccessHandle, AtomicAccessor, access::Access},
    transactions::Transaction,
};

pub struct AtomicAccess<T: Transaction, A: AtomicAccessor> {
    accessor: AtomicWeak<A>,
    accesses: Vec<AtomicOptionArc<Access<T, A>>>,
    pending_accesses: AtomicU64,
}

impl<T: Transaction, A: AtomicAccessor> AtomicAccess<T, A> {
    pub fn new(size: usize) -> Self {
        Self {
            pending_accesses: AtomicU64::new(size as u64),
            accessor: AtomicWeak::default(),
            accesses: (0..size).map(|_| AtomicOptionArc::empty()).collect(),
        }
    }

    pub fn publish_accessor(self: &Arc<Self>, accessor: &Arc<A>) {
        self.accessor.store(Arc::downgrade(accessor));

        if self.pending_accesses.load(Ordering::Acquire) == 0 {
            accessor.available();
        }
    }

    pub fn handles(&self) -> Vec<AccessHandle<T>> {
        self.accesses
            .iter()
            .map(|access| {
                if let Some(access) = access.load() {
                    access
                        .loaded_state()
                        .expect("must exist")
                        .cow_handle(access.metadata().clone())
                } else {
                    panic!("Access not available");
                }
            })
            .collect()
    }

    pub fn release(&self, handles: Vec<AccessHandle<T>>) {
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
                accessor.available();
            }
        }
    }
}
