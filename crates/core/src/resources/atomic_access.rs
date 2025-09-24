use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    atomic::AtomicWeak,
    resources::{AtomicAccessor, access::Access},
    transactions::Transaction,
};

pub struct AtomicAccess<T: Transaction, A: AtomicAccessor> {
    accessor: AtomicWeak<A>,
    resources: Vec<AtomicWeak<Access<T, A>>>,
    pending_resources: AtomicU64,
}

impl<T: Transaction, A: AtomicAccessor> AtomicAccess<T, A> {
    pub fn new(size: usize) -> Self {
        Self {
            pending_resources: AtomicU64::new(size as u64),
            accessor: AtomicWeak::default(),
            resources: (0..size).map(|_| AtomicWeak::default()).collect(),
        }
    }

    pub fn init_consumer(self: &Arc<Self>, consumer: &Arc<A>) {
        self.accessor.store(Arc::downgrade(consumer));

        if self.pending_resources.load(Ordering::Acquire) == 0 {
            consumer.available();
        }
    }

    pub fn release(&self) {
        for resource in &self.resources {
            if let Some(resource) = resource.load().upgrade() {
                resource.write(resource.loaded_state().unwrap());
            }
        }
    }

    pub(crate) fn notify(self: &Arc<Self>, resource: Arc<Access<T, A>>) {
        self.resources
            .get(resource.atomic_ref().1)
            .unwrap()
            .store(Arc::downgrade(&resource));

        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(consumer) = self.accessor.load().upgrade() {
                consumer.available();
            }
        }
    }
}
