use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_interface::{ReadStore, Store, WriteStore};
use kas_l2_storage_manager::{ReadCmd, WriteCmd};

use crate::{ResourceAccess, RuntimeBatch, StateDiff, vm::VM};

pub enum Read<S: Store<StateSpace = StateSpace>, V: VM> {
    LatestData(ResourceAccess<S, V>),
}

impl<S: Store<StateSpace = StateSpace>, V: VM> ReadCmd<StateSpace> for Read<S, V> {
    fn exec<RS: ReadStore<StateSpace = StateSpace>>(&self, store: &RS) {
        match self {
            Read::LatestData(resource_access) => resource_access.read_latest_data(store),
        }
    }
}

pub enum Write<S: Store<StateSpace = StateSpace>, V: VM> {
    StateDiff(StateDiff<S, V>),
    CommitBatch(RuntimeBatch<S, V>),
}

impl<S: Store<StateSpace = StateSpace>, V: VM> WriteCmd<StateSpace> for Write<S, V> {
    fn exec<WS: WriteStore<StateSpace = StateSpace>>(&self, store: &mut WS) {
        match self {
            Write::StateDiff(state_diff) => state_diff.write(store),
            Write::CommitBatch(batch) => batch.commit(store),
        }
    }

    fn done(self) {
        match self {
            Write::StateDiff(state_diff) => state_diff.write_done(),
            Write::CommitBatch(batch) => batch.commit_done(),
        }
    }
}
