use std::collections::HashMap;
use std::sync::Arc;

use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, AccessMetadata};
use move_binary_format::errors::VMError;
use move_core_types::{
    account_address::AccountAddress, effects::Op, identifier::Identifier,
    language_storage::ModuleId,
};
use move_vm_runtime::{move_vm::MoveVM, session::SerializedReturnValues};
use move_vm_types::{gas::UnmeteredGasMeter, loaded_data::runtime_types::Type};

use crate::{
    instruction::{Instruction, MethodArg},
    module_resolver::ModuleResolver,
    object_id::ObjectId,
    transaction::Transaction,
};

pub struct VM {
    move_vm: Arc<MoveVM>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            move_vm: Arc::new(MoveVM::new([]).unwrap()),
        }
    }

    pub fn process_transaction(
        &self,
        tx: &Transaction,
        resources: &mut [AccessHandle<RocksDbStore, Transaction>],
    ) -> Result<(), VMError> {
        let mut modules = ModuleResolver::new();
        let mut loaded_data = vec![];
        let mut resources_by_id = HashMap::new();
        for resource in resources {
            match resource.access_metadata().id() {
                ObjectId::Data(_) => loaded_data.push(resource.state().data.clone()),
                ObjectId::Module(module_id) => {
                    if !resource.is_new() {
                        modules.add_module(module_id.clone(), resource.state().data.clone())
                    }
                    resources_by_id.insert(module_id, resource);
                }
            }
        }

        match &tx.instruction {
            Instruction::MethodCall {
                module_ref,
                function_name,
                ty_args,
                args,
            } => {
                let module_id = modules.module_id(*module_ref);
                let args = args
                    .into_iter()
                    .map(|arg| match arg {
                        MethodArg::DataRef(index) => loaded_data[*index].clone(),
                        MethodArg::MoveValue(value) => bcs::to_bytes(&value).unwrap(),
                    })
                    .collect();

                let res = self.call_method(module_id, function_name, ty_args.clone(), args)?;
                res.mutable_reference_outputs;
            }
            Instruction::PublishModules { modules, sender } => {
                for (module_id, op) in self.publish_modules(modules.clone(), sender.clone())? {
                    let Some(resource) = resources_by_id.get_mut(&module_id) else {
                        panic!("no resource found for published module {:?}", module_id);
                    };

                    match op {
                        Op::New(data) | Op::Modify(data) => {
                            resource.state_mut().data = data;
                        }
                        Op::Delete => panic!("published module cannot be deleted"),
                    }
                }
            }
        }

        Ok(())
    }

    fn call_method(
        &self,
        module_id: &ModuleId,
        function_name: &Identifier,
        ty_args: Vec<Type>,
        args: Vec<Vec<u8>>,
    ) -> Result<SerializedReturnValues, VMError> {
        let mut session = self.move_vm.new_session(ModuleResolver::new());

        let result = session.execute_entry_function(
            module_id,
            function_name,
            ty_args.clone(),
            args,
            &mut UnmeteredGasMeter,
        )?;

        session.finish().0.expect("session should be finished");

        Ok(result)
    }

    fn publish_modules(
        &self,
        modules_bytes: Vec<Vec<u8>>,
        sender: AccountAddress,
    ) -> Result<Vec<(ModuleId, Op<Vec<u8>>)>, VMError> {
        let mut session = self.move_vm.new_session(ModuleResolver::new());
        session.publish_module_bundle(modules_bytes, sender, &mut UnmeteredGasMeter)?;
        Ok(session.finish().0?.into_modules().collect())
    }
}
