use kas_l2_storage::{Store, WriteCmd, WriteStore};

use crate::{Batch, StateDiff, Transaction, storage::runtime_state::RuntimeState};

pub enum Write<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    StateDiff(StateDiff<S, T>),
    PointerFlip(Batch<S, T>),
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> WriteCmd<RuntimeState> for Write<S, Tx> {
    fn exec<WS: WriteStore<StateSpace = RuntimeState>>(&self, store: &mut WS) {
        match self {
            Write::StateDiff(state_diff) => state_diff.write(store),
            Write::PointerFlip(batch) => batch.write_pointer_flip(store),
        }
    }

    fn mark_committed(self) {
        match self {
            Write::StateDiff(state_diff) => state_diff.write_done(),
            Write::PointerFlip(batch) => batch.mark_committed(),
        }
    }
}
