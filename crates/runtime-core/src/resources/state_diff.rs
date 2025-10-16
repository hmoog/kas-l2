use std::sync::Arc;

use kas_l2_atomic::AtomicOptionArc;
use kas_l2_runtime_macros::smart_pointer;
use kas_l2_storage::{Storage, Store, WriteStore};

use crate::{
    BatchRef, ResourceId, RuntimeState, State, Transaction,
    storage::{read_cmd::Read, write_cmd::Write},
};

#[smart_pointer]
pub struct StateDiff<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    storage: Storage<S, Read<S, T>, Write<S, T>>,
    batch: BatchRef<S, T>,
    resource_id: T::ResourceId,
    read_state: AtomicOptionArc<State<T>>,
    written_state: AtomicOptionArc<State<T>>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> StateDiff<S, T> {
    pub fn new(
        storage: Storage<S, Read<S, T>, Write<S, T>>,
        batch: BatchRef<S, T>,
        resource_id: T::ResourceId,
    ) -> Self {
        Self(Arc::new(StateDiffData {
            storage,
            batch,
            resource_id,
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
        }))
    }

    pub fn resource_id(&self) -> &T::ResourceId {
        &self.resource_id
    }

    pub fn read_state(&self) -> Arc<State<T>> {
        self.read_state.load().expect("read state unknown")
    }

    pub fn written_state(&self) -> Arc<State<T>> {
        self.written_state.load().expect("written state unknown")
    }

    pub(crate) fn set_read_state(&self, state: Arc<State<T>>) {
        self.read_state.store(Some(state))
    }

    pub(crate) fn set_written_state(&self, state: Arc<State<T>>) {
        self.written_state.store(Some(state));

        if let Some(batch) = self.batch.upgrade() {
            batch.increase_pending_writes();
            self.storage.submit_write(Write::StateDiff(self.clone()));
        }
    }

    fn index(&self) -> u64 {
        1
    }

    pub(crate) fn write_to<WS: WriteStore<StateSpace = RuntimeState>>(&self, store: &WS) {
        let id: Vec<u8> = self.resource_id().clone().to_bytes();
        let latest_version = self.index().to_le_bytes().to_vec();
        store
            .put(
                RuntimeState::LatestDataPointers,
                &id[..],
                &latest_version[..],
            )
            .expect("failed to update latest data pointers");

        let mut versioned_id = latest_version;
        versioned_id.extend_from_slice(&id[..]);
        store
            .put(RuntimeState::Data, &versioned_id[..], &[])
            .expect("failed to write state");
        // TODO: WRITE LATEST STATE
        // TODO: WRITE STATE_DIFF FOR ROLLBACK
    }

    pub(crate) fn mark_committed(self) {
        if let Some(batch) = self.batch.upgrade() {
            batch.decrease_pending_writes();
        }
    }
}
