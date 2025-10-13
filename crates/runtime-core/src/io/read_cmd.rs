use kas_l2_io_core::ReadableKVStore;
use kas_l2_io_manager::ReadCmd;

use crate::{
    Transaction, io::runtime_state::RuntimeState, resources::resource_access::ResourceAccess,
};

pub enum Read<Tx: Transaction> {
    ResourceAccess(ResourceAccess<Tx>),
}

impl<Tx: Transaction> ReadCmd<RuntimeState> for Read<Tx> {
    fn exec<S: ReadableKVStore<Namespace = RuntimeState>>(&self, store: &S) {
        match self {
            Read::ResourceAccess(access) => access.load_from_storage(store),
        }
    }
}
