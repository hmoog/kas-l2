use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_atomic::{AtomicOptionArc, AtomicWeak};
use kas_l2_macros::smart_pointer;
use kas_l2_storage::{ReadStore, Storage, Store};

use crate::{
    AccessMetadata, AccessType, RuntimeTxRef, StateDiff, Transaction, VersionedState,
    storage::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

#[smart_pointer(deref(metadata))]
pub struct ResourceAccess<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    metadata: T::AccessMetadata,
    is_batch_head: AtomicBool,
    is_batch_tail: AtomicBool,
    tx: RuntimeTxRef<S, T>,
    state_diff: StateDiff<S, T>,
    read_state: AtomicOptionArc<VersionedState<T>>,
    written_state: AtomicOptionArc<VersionedState<T>>,
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> ResourceAccess<S, T> {
    #[inline(always)]
    pub fn metadata(&self) -> &T::AccessMetadata {
        &self.metadata
    }

    #[inline(always)]
    pub fn read_state(&self) -> Arc<VersionedState<T>> {
        self.read_state.load().expect("read state unavailable")
    }

    #[inline(always)]
    pub fn written_state(&self) -> Arc<VersionedState<T>> {
        self.written_state
            .load()
            .expect("written state unavailable")
    }

    #[inline(always)]
    pub fn is_batch_head(&self) -> bool {
        self.is_batch_head.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn is_batch_tail(&self) -> bool {
        self.is_batch_tail.load(Ordering::Relaxed)
    }

    pub(crate) fn new(
        metadata: T::AccessMetadata,
        tx: RuntimeTxRef<S, T>,
        state_diff: StateDiff<S, T>,
        prev: Option<Self>,
    ) -> Self {
        Self(Arc::new(ResourceAccessData {
            metadata,
            is_batch_head: AtomicBool::new(match &prev {
                Some(prev) if prev.state_diff == state_diff => {
                    prev.is_batch_tail.store(false, Ordering::Relaxed);
                    false
                }
                _ => true,
            }),
            is_batch_tail: AtomicBool::new(true),
            tx,
            state_diff,
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
            prev: AtomicOptionArc::new(prev.map(|p| p.0)),
            next: AtomicWeak::default(),
        }))
    }

    pub(crate) fn init(&self, storage: &Storage<S, Read<S, T>, Write<S, T>>) {
        match self.prev.load() {
            Some(prev) => {
                prev.next.store(Arc::downgrade(&self.0));
                if let Some(written_state) = prev.written_state.load() {
                    self.set_read_state(written_state);
                }
            }
            None => storage.submit_read(Read::ResourceAccess(self.clone())),
        }
    }

    pub(crate) fn read_from_store<R: ReadStore<StateSpace = RuntimeState>>(&self, store: &R) {
        self.set_read_state(Arc::new(VersionedState::from_latest_data(
            store,
            self.metadata.id(),
        )));
    }

    pub(crate) fn tx(&self) -> &RuntimeTxRef<S, T> {
        &self.tx
    }

    pub(crate) fn state_diff(&self) -> StateDiff<S, T> {
        self.state_diff.clone()
    }

    pub(crate) fn set_read_state(&self, state: Arc<VersionedState<T>>) {
        if self.read_state.publish(state.clone()) {
            drop(self.prev.take()); // drop the previous reference to allow cleanup

            if self.is_batch_head() {
                self.state_diff.set_read_state(state.clone());
            }

            if self.access_type() == AccessType::Read {
                self.set_written_state(state);
            }

            if let Some(tx) = self.tx.upgrade() {
                tx.decrease_pending_resources();
            }
        }
    }

    pub(crate) fn set_written_state(&self, state: Arc<VersionedState<T>>) {
        if self.written_state.publish(state.clone()) {
            if self.is_batch_tail() {
                self.state_diff.set_written_state(state.clone());
            }

            if let Some(next) = self.next.load().upgrade() {
                Self(next).set_read_state(state)
            }
        }
    }
}
