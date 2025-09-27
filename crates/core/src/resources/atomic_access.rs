use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    atomic::{AtomicOptionArc, AtomicWeak},
    resources::{AccessHandle, AccessMetadata, AccessType, AtomicAccessor, access::Access},
    transactions::Transaction,
};

pub struct AtomicAccess<T: Transaction, A: AtomicAccessor> {
    accessor: AtomicWeak<A>,
    resources: Vec<AtomicOptionArc<Access<T, A>>>,
    pending_resources: AtomicU64,
}

impl<T: Transaction, A: AtomicAccessor> AtomicAccess<T, A> {
    pub fn new(resources: Vec<Arc<Access<T, A>>>) -> Self {
        Self {
            accessor: AtomicWeak::default(),
            pending_resources: AtomicU64::new(resources.len() as u64),
            resources: resources
                .into_iter()
                .map(|y| AtomicOptionArc::new(Some(y)))
                .collect(),
        }
    }

    pub fn load_missing<F: Fn(Arc<Access<T, A>>)>(self: Arc<Self>, loader: F) -> Arc<Self> {
        for access in self.resources.iter() {
            let access = access.load().unwrap();
            match access.prev_access() {
                Some(prev) => prev.link_next_access(access),
                None => loader(access),
            }
        }
        self
    }

    pub fn init_accessor(self: &Arc<Self>, accessor: &Arc<A>) {
        self.accessor.store(Arc::downgrade(accessor));

        if self.pending_resources.load(Ordering::Acquire) == 0 {
            accessor.notify();
        }
    }

    pub fn consume<F: FnOnce(&mut [AccessHandle<T>])>(&self, processor: F) {
        let resources = self
            .resources
            .iter()
            .filter_map(AtomicOptionArc::take)
            .collect::<Vec<_>>();
        assert_eq!(
            resources.len(),
            self.resources.len(),
            "Not all accesses are ready"
        );

        let mut handles = resources
            .iter()
            .map(|access| {
                AccessHandle::new(
                    access.loaded_state().expect("missing state"),
                    access.metadata(),
                )
            })
            .collect::<Vec<_>>();

        processor(&mut handles);

        for (handle, access) in handles.into_iter().zip(resources.iter()) {
            if handle.access_type() == AccessType::Write {
                access.publish_written_state(handle.commit());
            }
        }
    }

    pub(crate) fn decrease_pending_resources(self: &Arc<Self>) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(accessor) = self.accessor.load().upgrade() {
                accessor.notify();
            }
        }
    }
}
