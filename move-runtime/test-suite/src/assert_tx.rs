use vprogs_move_runtime_vm::TransactionEffects;
use vprogs_scheduling_scheduler::{RuntimeTx, VmInterface};
use vprogs_storage_state::StateSpace;
use vprogs_storage_types::Store;

use crate::AssertTxEffect;

pub struct AssertTx {
    pub tx_index: usize,
    pub effects: Vec<AssertTxEffect>,
}

impl AssertTx {
    pub fn new(tx_index: usize, effects: Vec<AssertTxEffect>) -> Self {
        Self { tx_index, effects }
    }
}

pub trait AssertTxExt {
    fn assert(&self, assertions: &[AssertTx]);
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface<TransactionEffects = TransactionEffects>>
    AssertTxExt for &[RuntimeTx<S, V>]
{
    fn assert(&self, assertions: &[AssertTx]) {
        for tx_assertions in assertions {
            let effects = self.get(tx_assertions.tx_index).expect("invalid tx index").effects();
            for assertion in &tx_assertions.effects {
                assertion.assert(&effects);
            }
        }
    }
}
