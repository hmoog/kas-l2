use std::sync::Arc;

use kas_l2_core_atomics::AtomicOptionArc;
use kas_l2_core_macros::smart_pointer;
use kas_l2_storage_manager::{Store, WriteStore};

use crate::{BatchRef, RuntimeState, VersionedState, Vm, storage::cmd::Write};

#[smart_pointer]
pub struct StateDiff<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    batch: BatchRef<S, VM>,
    resource_id: VM::ResourceId,
    read_state: AtomicOptionArc<VersionedState<VM>>,
    written_state: AtomicOptionArc<VersionedState<VM>>,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> StateDiff<S, VM> {
    pub fn new(batch: BatchRef<S, VM>, resource_id: VM::ResourceId) -> Self {
        Self(Arc::new(StateDiffData {
            batch,
            resource_id,
            read_state: AtomicOptionArc::empty(),
            written_state: AtomicOptionArc::empty(),
        }))
    }

    pub fn resource_id(&self) -> &VM::ResourceId {
        &self.resource_id
    }

    pub fn read_state(&self) -> Arc<VersionedState<VM>> {
        self.read_state.load().expect("read state unknown")
    }

    pub fn written_state(&self) -> Arc<VersionedState<VM>> {
        self.written_state.load().expect("written state unknown")
    }

    pub(crate) fn set_read_state(&self, state: Arc<VersionedState<VM>>) {
        self.read_state.store(Some(state))
    }

    pub(crate) fn set_written_state(&self, state: Arc<VersionedState<VM>>) {
        self.written_state.store(Some(state));
        if let Some(batch) = self.batch.upgrade() {
            batch.submit_write(Write::StateDiff(self.clone()));
        }
    }

    pub(crate) fn write<WS: WriteStore<StateSpace = RuntimeState>>(&self, store: &mut WS) {
        let Some(batch) = self.batch.upgrade() else {
            panic!("batch must be known at write time");
        };
        let Some(read_state) = self.read_state.load() else {
            panic!("read_state must be known at write time");
        };
        let Some(written_state) = self.written_state.load() else {
            panic!("written_state must be known at write time");
        };

        written_state.write_data(store);
        read_state.write_rollback_ptr(store, batch.index());
    }

    pub(crate) fn write_done(self) {
        if let Some(batch) = self.batch.upgrade() {
            batch.decrease_pending_writes();
        }
    }
}
