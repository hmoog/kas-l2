use std::{sync::Arc};

use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle};
use move_binary_format::errors::VMError;
use move_vm_runtime::{move_vm::MoveVM};

use crate::{
    instructions::instruction::{Instruction},
    transaction::Transaction,
};
use crate::execution_context::ExecutionContext;

pub struct VM {
    move_vm: Arc<MoveVM>,
}

impl VM {
    pub fn process_transaction(
        &self,
        tx: &Transaction,
        resources: &mut [AccessHandle<RocksDbStore, Transaction>],
    ) -> Result<(), VMError> {
        let ctx = ExecutionContext::new(&self.move_vm, resources);

        match &tx.instruction {
            Instruction::MethodCall(function_call) => function_call.execute(ctx),
            Instruction::PublishModules(publish_modules) => publish_modules.execute(ctx),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self {
            move_vm: Arc::new(MoveVM::new([]).unwrap()),
        }
    }
}
