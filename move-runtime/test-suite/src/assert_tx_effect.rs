use std::sync::Arc;

use kas_l2_move_runtime_vm::TransactionEffects;
use move_core_types::runtime_value::MoveValue;

use crate::AssertRetValRef;

pub enum AssertTxEffect {
    ReturnValue { reference: AssertRetValRef, expected: MoveValue },
}

impl AssertTxEffect {
    pub fn assert(&self, effects: &Arc<TransactionEffects>) {
        match self {
            AssertTxEffect::ReturnValue { reference, expected } => {
                assert_eq!(
                    &effects[reference.0][reference.1].0,
                    &bcs::to_bytes(expected).unwrap(),
                    "assertion failed for return value at index {:?}",
                    reference
                );
            }
        }
    }
}
