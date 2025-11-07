use kas_l2_storage_manager::{ReadCmd, ReadStore, Store, WriteCmd, WriteStore};

use crate::{
    Batch, StateDiff, resources::resource_access::ResourceAccess,
    storage::runtime_state::RuntimeState, vm::VM,
};

pub enum Read<S: Store<StateSpace = RuntimeState>, V: VM> {
    LatestData(ResourceAccess<S, V>),
}

impl<S: Store<StateSpace = RuntimeState>, V: VM> ReadCmd<RuntimeState> for Read<S, V> {
    fn exec<RS: ReadStore<StateSpace = RuntimeState>>(&self, store: &RS) {
        match self {
            Read::LatestData(resource_access) => resource_access.read_latest_data(store),
        }
    }
}

pub enum Write<S: Store<StateSpace = RuntimeState>, V: VM> {
    StateDiff(StateDiff<S, V>),
    CommitBatch(Batch<S, V>),
}

impl<S: Store<StateSpace = RuntimeState>, V: VM> WriteCmd<RuntimeState> for Write<S, V> {
    fn exec<WS: WriteStore<StateSpace = RuntimeState>>(&self, store: &mut WS) {
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
