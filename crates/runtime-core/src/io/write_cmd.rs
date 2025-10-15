use kas_l2_storage::{WriteCmd, WriteStore};

use crate::{Batch, StateDiff, Transaction, io::runtime_state::RuntimeState};

pub enum Write<T: Transaction> {
    StateDiff(StateDiff<T>),
    Batch(Batch<T>),
}

impl<Tx: Transaction> WriteCmd<RuntimeState> for Write<Tx> {
    fn exec<S: WriteStore<StateSpace = RuntimeState>>(&self, _store: &S) {
        match self {
            Write::StateDiff(_state_diff) => {}
            Write::Batch(_batch) => {}
        }
    }
}
