use std::{collections::HashMap, sync::Arc};

use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, AccessMetadata};
use move_binary_format::errors::VMError;
use move_core_types::{
    account_address::AccountAddress, effects::Op, identifier::Identifier,
    language_storage::ModuleId,
};
use move_core_types::vm_status::StatusCode::TRANSACTION_EXPIRED;
use move_vm_runtime::{move_vm::MoveVM, session::SerializedReturnValues};
use move_vm_types::{gas::UnmeteredGasMeter, loaded_data::runtime_types::Type};

use crate::{
    instructions::instruction::{Instruction},
    module_resolver::ModuleResolver,
    object_id::ObjectId,
    transaction::Transaction,
};
use crate::instructions::function_call::FunctionCallArg;
use crate::transaction_context::TransactionContext;

pub struct VM {
    move_vm: Arc<MoveVM>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            move_vm: Arc::new(MoveVM::new([]).unwrap()),
        }
    }

    pub fn process_transaction<'a>(
        &self,
        tx: &'a Transaction,
        resources: &'a mut [AccessHandle<'a, RocksDbStore, Transaction>],
    ) -> Result<(), VMError> {
        let ctx = TransactionContext::new(&self.move_vm, resources);

        match &tx.instruction {
            Instruction::MethodCall(function_call) => function_call.execute(ctx),
            Instruction::PublishModules(publish_modules) => publish_modules.execute(ctx),
        }
    }
}
