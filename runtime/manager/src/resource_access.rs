use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use kas_l2_core_atomics::{AtomicOptionArc, AtomicWeak};
use kas_l2_core_macros::smart_pointer;
use kas_l2_runtime_state::{StateSpace, VersionedState};
use kas_l2_runtime_types::{AccessMetadata, AccessType};
use kas_l2_storage_manager::StorageManager;
use kas_l2_storage_types::{ReadStore, Store};
use tracing::trace;

use crate::{Read, RuntimeTxRef, StateDiff, Write, vm_interface::VmInterface};

#[smart_pointer(deref(metadata))]
pub struct ResourceAccess<S: Store<StateSpace = StateSpace>, V: VmInterface> {
    metadata: V::AccessMetadata,
    is_batch_head: AtomicBool,
    is_batch_tail: AtomicBool,
    tx: RuntimeTxRef<S, V>,
    state_diff: StateDiff<S, V>,
    read_state: AtomicOptionArc<VersionedState<V::ResourceId, V::Ownership>>,
    written_state: AtomicOptionArc<VersionedState<V::ResourceId, V::Ownership>>,
    prev: AtomicOptionArc<Self>,
    next: AtomicWeak<Self>,
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> ResourceAccess<S, V> {
    #[inline(always)]
    pub fn metadata(&self) -> &V::AccessMetadata {
        &self.metadata
    }

    #[inline(always)]
    pub fn read_state(&self) -> Arc<VersionedState<V::ResourceId, V::Ownership>> {
        self.read_state.load().expect("read state unknown")
    }

    #[inline(always)]
    pub fn written_state(&self) -> Arc<VersionedState<V::ResourceId, V::Ownership>> {
        self.written_state.load().expect("written state unknown")
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
        metadata: V::AccessMetadata,
        tx: RuntimeTxRef<S, V>,
        state_diff: StateDiff<S, V>,
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

    pub(crate) fn connect(&self, storage: &StorageManager<S, Read<S, V>, Write<S, V>>) {
        match self.prev.load() {
            Some(prev) => {
                trace!(is_batch_head = self.is_batch_head(), "connecting to previous access");
                prev.next.store(Arc::downgrade(&self.0));
                if let Some(written_state) = prev.written_state.load() {
                    trace!("previous written state available, setting read state immediately");
                    self.set_read_state(written_state);
                } else {
                    trace!("previous written state not yet available, will wait");
                }
            }
            None => {
                trace!(is_batch_head = self.is_batch_head(), "no previous access, submitting storage read");
                storage.submit_read(Read::LatestData(self.clone()));
            }
        }
    }

    pub(crate) fn read_latest_data<R: ReadStore<StateSpace = StateSpace>>(&self, store: &R) {
        self.set_read_state(Arc::new(VersionedState::from_latest_data(store, self.metadata.id())));
    }

    pub(crate) fn tx(&self) -> &RuntimeTxRef<S, V> {
        &self.tx
    }

    pub(crate) fn state_diff(&self) -> StateDiff<S, V> {
        self.state_diff.clone()
    }

    pub(crate) fn set_read_state(&self, state: Arc<VersionedState<V::ResourceId, V::Ownership>>) {
        if self.read_state.publish(state.clone()) {
            trace!(
                is_batch_head = self.is_batch_head(),
                access_type = ?self.access_type(),
                "read state set"
            );
            drop(self.prev.take()); // drop the previous reference to allow cleanup

            if self.is_batch_head() {
                self.state_diff.set_read_state(state.clone());
            }

            if self.access_type() == AccessType::Read {
                self.set_written_state(state);
            }

            if let Some(tx) = self.tx.upgrade() {
                trace!("notifying tx of resolved resource");
                tx.decrease_pending_resources();
            }
        } else {
            trace!("read state already published, skipping");
        }
    }

    pub(crate) fn set_written_state(
        &self,
        state: Arc<VersionedState<V::ResourceId, V::Ownership>>,
    ) {
        if self.written_state.publish(state.clone()) {
            trace!(is_batch_tail = self.is_batch_tail(), "written state set");
            if self.is_batch_tail() {
                self.state_diff.set_written_state(state.clone());
            }

            if let Some(next) = self.next.load().upgrade() {
                trace!("propagating written state to next access in chain");
                Self(next).set_read_state(state)
            }
        }
    }
}
