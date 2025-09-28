use std::{
    ops::Deref,
    sync::{Arc, Weak},
};

use kas_l2_atomic::{AtomicOptionArc, AtomicWeak};

use crate::{
    AccessType, Resources, State, access_metadata::AccessMetadata, transaction::Transaction,
};

pub(crate) struct Resource<T: Transaction> {
    resources: Weak<Resources<T>>,
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
    read_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
    access_metadata: T::AccessMetadata,
}

impl<T: Transaction> Resource<T> {
    pub(crate) fn new(
        resources: Weak<Resources<T>>,
        prev: Option<Arc<Self>>,
        access_metadata: T::AccessMetadata,
    ) -> Arc<Self> {
        Arc::new(Self {
            resources,
            prev: AtomicOptionArc::new(prev),
            next: AtomicWeak::default(),
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
            access_metadata,
        })
    }

    pub(crate) fn belongs_to(&self, resources: &Weak<Resources<T>>) -> bool {
        Weak::ptr_eq(&self.resources, resources)
    }

    pub(crate) fn prev(&self) -> Option<Arc<Self>> {
        self.prev.load()
    }

    pub(crate) fn set_next(&self, next: Arc<Self>) {
        if let Some(written_state) = self.written_state.take() {
            next.set_read_state(written_state);
        } else {
            self.next.store(Arc::downgrade(&next));
        }
    }

    pub(crate) fn read_state(&self) -> Arc<State<T>> {
        self.read_state.load().unwrap()
    }

    pub(crate) fn set_read_state(self: Arc<Self>, state: Arc<State<T>>) {
        drop(self.prev.take()); // allow previous resource to be dropped

        if self.access_type() == AccessType::Read {
            self.set_written_state(state.clone());
        }

        self.read_state.store(Some(state));

        self.resources
            .upgrade()
            .expect("resources missing")
            .decrease_pending_resources();
    }

    pub(crate) fn set_written_state(&self, state: Arc<State<T>>) {
        if let Some(next) = self.next.load().upgrade() {
            next.set_read_state(state)
        } else {
            self.written_state.store(Some(state))
        }
    }
}

impl<T: Transaction> Deref for Resource<T> {
    type Target = T::AccessMetadata;
    fn deref(&self) -> &Self::Target {
        &self.access_metadata
    }
}
