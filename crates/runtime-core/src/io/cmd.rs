use crate::{
    Batch, StateDiff, Transaction,
    io::{kv_store::KVStore, state_space::StateSpace},
    resources::resource_access::ResourceAccess,
};

pub enum ReadCommands<T: Transaction> {
    ReadState(ResourceAccess<T>),
}

impl<T: Transaction> ReadCommands<T> {
    pub fn exec<S: KVStore>(&self, store: &S) {
        match self {
            ReadCommands::ReadState(access) => {
                store.get(StateSpace::LatestDataPointers, &[]);
            }
        }
    }
}

pub enum WriteCmd<T: Transaction> {
    WriteState(StateDiff<T>),
    CommitBatch(Batch<T>),
}

impl<T: Transaction> WriteCmd<T> {
    pub fn exec<S: KVStore>(&self, _store: &S) {
        match self {
            WriteCmd::WriteState(_) => {}
            WriteCmd::CommitBatch(_) => {}
        }
    }
}