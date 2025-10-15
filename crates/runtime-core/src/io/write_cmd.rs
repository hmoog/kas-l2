use kas_l2_storage::{Store, WriteCmd, WriteStore};

use crate::{Batch, StateDiff, Transaction, io::runtime_state::RuntimeState};

pub enum Write<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    StateDiff(StateDiff<S, T>),
    Batch(Batch<S, T>),
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> WriteCmd<RuntimeState> for Write<S, Tx> {
    fn exec<WS: WriteStore<StateSpace = RuntimeState>>(&self, _store: &WS) {
        match self {
            Write::StateDiff(_state_diff) => {}
            Write::Batch(_batch) => {}
        }
    }

    fn commit(self) {
        match self {
            Write::StateDiff(state_diff) => state_diff.commit(),
            Write::Batch(_batch) => {}
        }
    }
}
