use kas_l2_storage::{ReadCmd, ReadStore, Store};

use crate::{
    Transaction, io::runtime_state::RuntimeState, resources::resource_access::ResourceAccess,
};

pub enum Read<S: Store<StateSpace = RuntimeState>, Tx: Transaction> {
    ResourceAccess(ResourceAccess<S, Tx>),
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> ReadCmd<RuntimeState> for Read<S, Tx> {
    fn exec<RS: ReadStore<StateSpace = RuntimeState>>(&self, store: &RS) {
        match self {
            Read::ResourceAccess(access) => access.load_from(store),
        }
    }
}
