use std::sync::Arc;

use kas_l2_atomic::{AtomicOptionArc, AtomicWeak};
use kas_l2_runtime_macros::smart_pointer;

use crate::{
    AccessMetadata, AccessType, ResourceProvider, RuntimeTx, RuntimeTxRef, State, Storage,
    Transaction,
};

#[smart_pointer(deref(access))]
pub struct AccessedResource<T: Transaction> {
    access: T::Access,
    tx_ref: RuntimeTxRef<T>,
    read_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
}

impl<T: Transaction> AccessedResource<T> {
    pub fn read_state(&self) -> Arc<State<T>> {
        self.read_state.load().expect("read state missing")
    }

    pub fn written_state(&self) -> Arc<State<T>> {
        self.written_state.load().expect("written state missing")
    }

    pub(crate) fn new(access: T::Access, tx_ref: RuntimeTxRef<T>, prev: Option<Self>) -> Self {
        Self(Arc::new(AccessedResourceData {
            access,
            tx_ref,
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
            prev: AtomicOptionArc::new(prev.map(|p| p.0)),
            next: AtomicWeak::default(),
        }))
    }

    pub(crate) fn init<S: Storage<T::ResourceId>>(&self, resources: &mut ResourceProvider<T, S>) {
        match self.prev.load() {
            Some(prev) => {
                prev.next.store(Arc::downgrade(&self.0));

                if let Some(written_state) = prev.written_state.load() {
                    self.set_read_state(written_state);
                }
            }
            None => resources.load_from_storage(self),
        }
    }

    pub(crate) fn tx(&self) -> RuntimeTx<T> {
        self.tx_ref.upgrade().expect("tx was dropped")
    }

    pub(crate) fn tx_ref(&self) -> &RuntimeTxRef<T> {
        &self.tx_ref
    }

    pub(crate) fn set_read_state(&self, state: Arc<State<T>>) {
        if self.read_state.publish(state.clone()) {
            drop(self.prev.take()); // drop the previous reference to allow cleanup

            if self.access_type() == AccessType::Read {
                self.set_written_state(state);
            }

            self.tx().decrease_pending_resources();
        }
    }

    pub(crate) fn set_written_state(&self, state: Arc<State<T>>) {
        if self.written_state.publish(state.clone()) {
            if let Some(next) = self.next.load().upgrade() {
                Self(next).set_read_state(state)
            }
        }
    }
}
