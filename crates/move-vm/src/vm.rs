use move_binary_format::errors::VMResult;
use move_vm_runtime::move_vm::MoveVM;

use crate::{
    execution_context::ExecutionContext, transaction::Transaction, type_alias::AccessHandle,
};

pub struct VM(MoveVM);

impl VM {
    pub fn new() -> Self {
        Self(MoveVM::new([]).expect("failed to initialize VM"))
    }

    pub fn process(&self, tx: &Transaction, res: &mut [AccessHandle]) -> VMResult<()> {
        tx.instruction.execute(ExecutionContext::new(&self.0, res))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
