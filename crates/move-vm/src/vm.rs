use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::AccessHandle;
use move_binary_format::errors::VMError;
use move_vm_runtime::move_vm::MoveVM;

use crate::{execution_context::ExecutionContext, transaction::Transaction};

pub struct VM(MoveVM);

impl VM {
    pub fn new() -> Self {
        Self(MoveVM::new([]).unwrap())
    }

    pub fn process_transaction(
        &self,
        tx: &Transaction,
        res: &mut [AccessHandle<RocksDbStore, Transaction>],
    ) -> Result<(), VMError> {
        tx.instruction.execute(ExecutionContext::new(&self.0, res))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}
