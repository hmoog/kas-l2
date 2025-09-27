use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    atomic::{AtomicOptionArc, AtomicWeak},
    resources::{AccessHandle, AccessMetadata, AccessType, Consumer, resource::Resource},
    transactions::Transaction,
};

pub struct Resources<T: Transaction, A: Consumer> {
    consumer: AtomicWeak<A>,
    resources: Vec<AtomicOptionArc<Resource<T, A>>>,
    pending_resources: AtomicU64,
}

impl<T: Transaction, A: Consumer> Resources<T, A> {
    pub fn init_consumer(self: Arc<Self>, consumer: &Arc<A>) {
        if self.pending_resources.load(Ordering::Acquire) == 0 {
            consumer.resources_available();
        } else {
            self.consumer.store(Arc::downgrade(consumer));
        }
    }

    pub fn consume<F: FnOnce(&mut [AccessHandle<T>])>(self: Arc<Self>, processor: F) {
        let resources: Vec<_> = self
            .resources
            .iter()
            .filter_map(AtomicOptionArc::take)
            .collect();
        assert_eq!(resources.len(), self.resources.len(), "missing resources");

        let mut handles: Vec<_> = resources
            .iter()
            .map(|access| AccessHandle::new(access.read_state(), &access))
            .collect();

        processor(&mut handles);

        for (handle, access) in handles.into_iter().zip(resources.iter()) {
            if handle.access_type() == AccessType::Write {
                access.set_written_state(handle.commit());
            }
        }
    }

    pub(crate) fn new(resources: Vec<Arc<Resource<T, A>>>) -> Self {
        let mut this = Self {
            consumer: AtomicWeak::default(),
            resources: Vec::new(),
            pending_resources: AtomicU64::new(resources.len() as u64),
        };

        for resource in resources {
            this.resources.push(AtomicOptionArc::new(Some(resource)));
        }

        this
    }

    pub(crate) fn init_resources<F: Fn(Arc<Resource<T, A>>)>(
        self: Arc<Self>,
        load: F,
    ) -> Arc<Self> {
        for resource in self.resources.iter() {
            let resource = resource.load().expect("missing resource");
            match resource.prev() {
                Some(prev) => prev.set_next(resource),
                None => load(resource),
            }
        }
        self
    }

    pub(crate) fn decrease_pending_resources(self: Arc<Self>) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(consumer) = self.consumer.load().upgrade() {
                consumer.resources_available();
            }
        }
    }
}
