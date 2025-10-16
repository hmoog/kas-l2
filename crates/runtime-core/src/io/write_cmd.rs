use kas_l2_storage::{Store, WriteCmd, WriteStore};

use crate::{Batch, StateDiff, Transaction, io::runtime_state::RuntimeState};

pub enum Write<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    StateDiff(StateDiff<S, T>),
    Batch(Batch<S, T>),
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> WriteCmd<RuntimeState> for Write<S, Tx> {
    fn exec<WS: WriteStore<StateSpace = RuntimeState>>(&self, store: &WS) {
        match self {
            Write::StateDiff(state_diff) => state_diff.write_to(store),
            Write::Batch(batch) => batch.write_to(store),
        }
    }

    fn mark_committed(self) {
        match self {
            Write::StateDiff(state_diff) => state_diff.mark_committed(),
            Write::Batch(batch) => batch.mark_committed(),
        }
    }
}
