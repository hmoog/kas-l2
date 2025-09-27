use std::{
    ops::Deref,
    sync::{Arc, Weak},
};

use crate::{
    atomic::{AtomicOptionArc, AtomicWeak},
    resources::{AccessType, AtomicAccess, AtomicAccessor, State, access_metadata::AccessMetadata},
    transactions::Transaction,
};

pub struct Access<T: Transaction, A: AtomicAccessor> {
    metadata: T::AccessMetadata,
    atomic_ref: (Weak<AtomicAccess<T, A>>, usize),
    loaded_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
    prev_access: AtomicOptionArc<Self>,
    next_access: AtomicWeak<Self>,
}

impl<T: Transaction, R: AtomicAccessor> Access<T, R> {
    pub fn new(
        metadata: T::AccessMetadata,
        atomic_ref: (Weak<AtomicAccess<T, R>>, usize),
        prev: Option<Arc<Self>>,
    ) -> Self {
        Self {
            metadata,
            atomic_ref,
            loaded_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
            prev_access: AtomicOptionArc::new(prev),
            next_access: AtomicWeak::default(),
        }
    }

    pub fn metadata(&self) -> &T::AccessMetadata {
        &self.metadata
    }

    pub fn atomic_ref(&self) -> &(Weak<AtomicAccess<T, R>>, usize) {
        &self.atomic_ref
    }

    pub fn loaded_state(&self) -> Option<Arc<State<T>>> {
        self.loaded_state.load()
    }

    pub fn written_state(&self) -> Option<Arc<State<T>>> {
        self.written_state.load()
    }

    pub fn prev_access(&self) -> Option<Arc<Self>> {
        self.prev_access.load()
    }

    pub fn next_access(&self) -> Option<Arc<Self>> {
        self.next_access.load().upgrade()
    }

    pub fn load_state(self: Arc<Self>, state: Arc<State<T>>) {
        drop(self.prev_access.take()); // allow the previous provider to be dropped

        if self.access_type() == AccessType::Read {
            self.publish_written_state(state.clone());
        }

        if self.loaded_state.publish(state) {
            if let Some(atomic_access) = self.atomic_ref.0.upgrade() {
                atomic_access.decrease_pending_resources();
            }
        }
    }

    pub fn publish_written_state(&self, state: Arc<State<T>>) {
        if let Some(next_access) = self.next_access.load().upgrade() {
            next_access.load_state(state)
        } else {
            self.written_state.store(Some(state))
        }
    }

    pub fn link_next_access(&self, next_access: Arc<Self>) {
        if let Some(state) = self.written_state() {
            next_access.load_state(state);
        } else {
            self.next_access.store(Arc::downgrade(&next_access));
        }
    }
}

impl<T: Transaction, C: AtomicAccessor> Deref for Access<T, C> {
    type Target = T::AccessMetadata;
    fn deref(&self) -> &Self::Target {
        self.metadata()
    }
}
