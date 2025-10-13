use kas_l2_io_core::WriteableKVStore;
use kas_l2_io_manager::WriteCmd as WriteCommand;

use crate::{Batch, StateDiff, Transaction, io::runtime_state::RuntimeState};

pub enum Write<T: Transaction> {
    StateDiff(StateDiff<T>),
    Batch(Batch<T>),
}

impl<Tx: Transaction> WriteCommand<RuntimeState> for Write<Tx> {
    fn exec<S: WriteableKVStore<Namespace = RuntimeState>>(&self, _store: &S) {
        match self {
            Write::StateDiff(_state_diff) => {}
            Write::Batch(_batch) => {}
        }
    }
}
