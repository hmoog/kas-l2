use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_atomic::{AtomicOptionArc, AtomicWeak};
use kas_l2_runtime_macros::smart_pointer;

use crate::{
    AccessMetadata, AccessType, ResourceProvider, RuntimeTxRef, State, StateDiffRef, Storage,
    Transaction,
};

#[smart_pointer(deref(metadata))]
pub struct ResourceAccess<T: Transaction> {
    metadata: T::AccessMetadata,
    tx: RuntimeTxRef<T>,
    state_diff: StateDiffRef<T>,
    is_batch_head: AtomicBool,
    is_batch_tail: AtomicBool,
    read_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
}

impl<T: Transaction> ResourceAccess<T> {
    pub fn metadata(&self) -> &T::AccessMetadata {
        &self.metadata
    }

    pub fn read_state(&self) -> Arc<State<T>> {
        self.read_state.load().expect("state unavailable")
    }

    pub fn written_state(&self) -> Arc<State<T>> {
        self.written_state.load().expect("state unavailable")
    }

    pub(crate) fn new(
        metadata: T::AccessMetadata,
        tx: RuntimeTxRef<T>,
        state_diff: StateDiffRef<T>,
        prev: Option<Self>,
    ) -> Self {
        Self(Arc::new(ResourceAccessData {
            metadata,
            tx,
            state_diff,
            is_batch_head: AtomicBool::default(),
            is_batch_tail: AtomicBool::default(),
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
            prev: AtomicOptionArc::new(prev.map(|p| p.0)),
            next: AtomicWeak::default(),
        }))
    }

    pub(crate) fn init<S: Storage<T::ResourceId>>(&self, resources: &mut ResourceProvider<T, S>) {
        match self.prev.load() {
            Some(prev) => {
                let prev = Self(prev);
                prev.next.store(Arc::downgrade(&self.0));

                if prev.state_diff == self.state_diff {
                    prev.is_batch_head.store(false, Ordering::Release);
                    self.is_batch_tail.store(true, Ordering::Release);
                } else {
                    self.is_batch_head.store(true, Ordering::Release);
                }

                if let Some(written_state) = prev.written_state.load() {
                    self.set_read_state(written_state);
                }
            }
            None => {
                self.is_batch_head.store(true, Ordering::Release);
                self.is_batch_tail.store(true, Ordering::Release);

                resources.load_from_storage(self)
            }
        }
    }

    pub(crate) fn tx(&self) -> &RuntimeTxRef<T> {
        &self.tx
    }

    pub(crate) fn state_diff(&self) -> StateDiffRef<T> {
        self.state_diff.clone()
    }

    pub(crate) fn set_read_state(&self, state: Arc<State<T>>) {
        if self.read_state.publish(state.clone()) {
            drop(self.prev.take()); // drop the previous reference to allow cleanup

            if self.is_batch_head.load(Ordering::Acquire) {
                if let Some(state_diff) = self.state_diff.upgrade() {
                    state_diff.set_read_state(state.clone());
                }
            }

            if self.access_type() == AccessType::Read {
                self.set_written_state(state);
            }

            if let Some(tx) = self.tx.upgrade() {
                tx.decrease_pending_resources();
            }
        }
    }

    pub(crate) fn set_written_state(&self, state: Arc<State<T>>) {
        if self.written_state.publish(state.clone()) {
            if self.is_batch_tail.load(Ordering::Acquire) {
                if let Some(state_diff) = self.state_diff.upgrade() {
                    state_diff.set_written_state(state.clone());
                }
            }

            if let Some(next) = self.next.load().upgrade() {
                Self(next).set_read_state(state)
            }
        }
    }
}
