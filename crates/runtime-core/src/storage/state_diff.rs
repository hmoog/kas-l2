use std::sync::Arc;

use kas_l2_atomic::AtomicOptionArc;
use kas_l2_runtime_macros::smart_pointer;
use kas_l2_storage::{Storage, Store, WriteStore};

use crate::{
    BatchRef, RuntimeState, Transaction, VersionedState,
    storage::{read_cmd::Read, write_cmd::Write},
};

#[smart_pointer]
pub struct StateDiff<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    storage: Storage<S, Read<S, T>, Write<S, T>>,
    batch: BatchRef<S, T>,
    resource_id: T::ResourceId,
    read_state: AtomicOptionArc<VersionedState<T>>,
    written_state: AtomicOptionArc<VersionedState<T>>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> StateDiff<S, T> {
    pub fn new(
        storage: &Storage<S, Read<S, T>, Write<S, T>>,
        batch: BatchRef<S, T>,
        resource_id: T::ResourceId,
    ) -> Self {
        Self(Arc::new(StateDiffData {
            storage: storage.clone(),
            batch,
            resource_id,
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
        }))
    }

    pub fn resource_id(&self) -> &T::ResourceId {
        &self.resource_id
    }

    pub fn read_state(&self) -> Arc<VersionedState<T>> {
        self.read_state.load().expect("read state unknown")
    }

    pub fn written_state(&self) -> Arc<VersionedState<T>> {
        self.written_state.load().expect("written state unknown")
    }

    pub(crate) fn set_read_state(&self, state: Arc<VersionedState<T>>) {
        self.read_state.store(Some(state))
    }

    pub(crate) fn set_written_state(&self, state: Arc<VersionedState<T>>) {
        self.written_state.store(Some(state));

        if let Some(batch) = self.batch.upgrade() {
            batch.increase_pending_writes();
            self.storage.submit_write(Write::StateDiff(self.clone()));
        }
    }

    pub(crate) fn write_to<WS: WriteStore<StateSpace = RuntimeState>>(&self, store: &WS) {
        let Some(read_state) = self.read_state.load() else {
            panic!("read_state must be known at write time");
        };
        let Some(written_state) = self.written_state.load() else {
            panic!("written_state must be known at write time");
        };

        let versioned_id = written_state.id();
        store.put(
            RuntimeState::Data,
            &versioned_id,
            &written_state.state.to_bytes(),
        );
        store.put(
            RuntimeState::Diffs,
            &versioned_id,
            &read_state.state.to_bytes(),
        );
    }

    pub(crate) fn mark_committed(self) {
        if let Some(batch) = self.batch.upgrade() {
            batch.decrease_pending_writes();
        }
    }
}
