use kas_l2_storage_manager::{ReadCmd, ReadStore, Store, WriteCmd, WriteStore};

use crate::{
    Batch, StateDiff, Transaction, resources::resource_access::ResourceAccess,
    storage::runtime_state::RuntimeState,
};

pub enum Read<S: Store<StateSpace = RuntimeState>, Tx: Transaction> {
    LatestData(ResourceAccess<S, Tx>),
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> ReadCmd<RuntimeState> for Read<S, Tx> {
    fn exec<RS: ReadStore<StateSpace = RuntimeState>>(&self, store: &RS) {
        match self {
            Read::LatestData(resource_access) => resource_access.read_latest_data(store),
        }
    }
}

pub enum Write<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    StateDiff(StateDiff<S, T>),
    CommitBatch(Batch<S, T>),
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> WriteCmd<RuntimeState> for Write<S, Tx> {
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
