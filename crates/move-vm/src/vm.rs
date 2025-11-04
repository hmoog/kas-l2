use std::sync::Arc;
use move_core_types::account_address::AccountAddress;
use move_core_types::effects::Op;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::runtime_value::{MoveStruct, MoveValue};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_types::gas::UnmeteredGasMeter;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::VMError;
use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, AccessMetadata};
use crate::instruction::{MethodArg, Instruction};
use crate::module_resolver::ModuleResolver;
use crate::object_id::ObjectId;
use crate::transaction::Transaction;

pub struct VM {
    move_vm: Arc<MoveVM>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            move_vm: Arc::new(MoveVM::new([]).unwrap()),
        }
    }

    pub fn process_transaction(&self, tx: &Transaction, resources: &mut [AccessHandle<RocksDbStore, Transaction>]) -> Result<(), ()> {
        let mut modules = ModuleResolver::new();
        let mut loaded_data = vec![];
        for resource in resources {
            match resource.access_metadata().id() {
                ObjectId::Data(_) => loaded_data.push(resource.state().data.clone()),
                ObjectId::Module(module_id) => modules.add_module(module_id, resource.state().data.clone()),
            }
        }

        match &tx.instruction {
            Instruction::MethodCall { module_ref, function_name, ty_args, args } => {
                let module_id = modules.module_id(*module_ref);
                let args = args.into_iter().map(|arg| match arg {
                    MethodArg::DataRef(index) => loaded_data[*index].clone(),
                    MethodArg::MoveValue(value) => bcs::to_bytes(&value).unwrap(),
                }).collect();

                self.call_method(module_id, function_name, ty_args.clone(), args);
            }
            Instruction::PublishModules { modules, sender } => {
                self.publish_modules(modules.clone(), sender.clone());
            }
        }

        Ok(())
    }

    fn call_method(&self, module_id: &ModuleId, function_name: &Identifier, ty_args: Vec<Type>, args: Vec<Vec<u8>>) {
        let mut session = self.move_vm.new_session(ModuleResolver::new());
        let res = session.execute_entry_function(
            module_id,
            function_name,
            ty_args.clone(),
            args,
            &mut UnmeteredGasMeter,
        );

        eprintln!("Execution result: {:?}", res);

        assert!(res.is_ok(), "MoveVM execution failed: {:?}", res);

        let (result , _store) = session.finish();

        match result {
            Ok(change_set) => {
                eprintln!("ChangeSet: {:?}", change_set);
            }
            Err(e) => panic!("Session finish failed: {:?}", e),
        }
    }

    fn publish_modules(&self, modules_bytes: Vec<Vec<u8>>, sender: AccountAddress) -> Result<Vec<(ModuleId, Op<Vec<u8>>)>, VMError> {
        let mut session = self.move_vm.new_session(ModuleResolver::new());

        session.publish_module_bundle(
            modules_bytes,
            sender,
            &mut UnmeteredGasMeter,
        )?;

        session.finish().0?.into_modules().collect()
    }
}