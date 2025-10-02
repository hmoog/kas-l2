use std::{ops::Deref, sync::Arc};

use kas_l2_atomic::{AtomicOptionArc, AtomicWeak};

use crate::{
    AccessMetadata, ScheduledTransaction, ScheduledTransactionRef, Transaction,
    resources::{access_type::AccessType, state::State},
};

pub struct ResourceAccess<T: Transaction> {
    access_metadata: T::AccessMetadata,
    parent: ScheduledTransactionRef<T>,
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
    read_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
}

impl<T: Transaction> ResourceAccess<T> {
    pub fn read_state(&self) -> Arc<State<T>> {
        self.read_state
            .load()
            .expect("tried to access resource before it was loaded")
    }

    pub fn written_state(&self) -> Arc<State<T>> {
        self.written_state
            .load()
            .expect("tried to access resource before it was written")
    }

    pub(crate) fn new(
        access_metadata: T::AccessMetadata,
        parent: ScheduledTransactionRef<T>,
        prev: Option<Arc<Self>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            access_metadata,
            parent,
            prev: AtomicOptionArc::new(prev),
            next: AtomicWeak::default(),
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
        })
    }

    pub(crate) fn init<F: FnOnce(&Arc<Self>)>(self: &Arc<Self>, load_state: F) {
        match self.prev.load() {
            Some(prev) => {
                if let Some(written_state) = prev.written_state.load() {
                    self.set_read_state(written_state);
                } else {
                    prev.next.store(Arc::downgrade(self));
                }
            }
            None => load_state(self),
        }
    }

    pub(crate) fn parent(&self) -> ScheduledTransaction<T> {
        self.parent.upgrade().expect("parent missing")
    }

    pub(crate) fn parent_eq(&self, parent: &ScheduledTransactionRef<T>) -> bool {
        self.parent == *parent
    }

    pub(crate) fn set_read_state(&self, state: Arc<State<T>>) {
        drop(self.prev.take()); // drop the previous reference to allow cleanup

        if self.access_type() == AccessType::Read {
            self.set_written_state(state.clone());
        }

        self.read_state.store(Some(state));

        self.parent().decrease_pending_resources();
    }

    pub(crate) fn set_written_state(&self, state: Arc<State<T>>) {
        if let Some(next) = self.next.load().upgrade() {
            next.set_read_state(state.clone())
        }

        self.written_state.store(Some(state));
    }
}

impl<T: Transaction> Deref for ResourceAccess<T> {
    type Target = T::AccessMetadata;
    fn deref(&self) -> &Self::Target {
        &self.access_metadata
    }
}
