use std::collections::HashMap;

use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, AccessMetadata, AccessType};
use move_binary_format::errors::VMResult;
use move_core_types::{effects::Op, runtime_value::MoveTypeLayout};
use move_vm_runtime::{
    move_vm::MoveVM,
    session::{SerializedReturnValues, Session},
};

use crate::{MethodCallArg, ModuleResolver, ObjectId, Transaction};

pub struct ExecutionContext<'a, 'v, 'res> {
    pub resources: &'res mut [AccessHandle<'a, RocksDbStore, Transaction>],
    pub input_objects: Vec<ObjectId>,
    pub mutations: HashMap<ObjectId, Op<Vec<u8>>>,
    pub last_args: Vec<ObjectId>,
    pub return_stack: Vec<Vec<(Vec<u8>, MoveTypeLayout)>>,
    pub session: Session<'a, 'v, ModuleResolver>,
}

impl<'elem, 'vm, 'res> ExecutionContext<'elem, 'vm, 'res> {
    pub fn new(
        vm: &'vm MoveVM,
        resources: &'res mut [AccessHandle<'elem, RocksDbStore, Transaction>],
    ) -> Self {
        let mut modules = ModuleResolver::default();
        let mut input_objects = Vec::with_capacity(resources.len());

        for resource in resources.iter() {
            let object_id = resource.access_metadata().id();

            if let ObjectId::Module(module_id) = &object_id {
                if !resource.is_new() {
                    modules.add_module(module_id.clone(), resource.state().data.clone())
                }
            }

            input_objects.push(object_id);
        }

        Self {
            resources,
            input_objects,
            last_args: Vec::new(),
            return_stack: Vec::new(),
            mutations: HashMap::new(),
            session: vm.new_session(modules),
        }
    }

    pub fn serialized_args(&mut self, args: &[MethodCallArg]) -> Vec<Vec<u8>> {
        self.last_args = Vec::with_capacity(args.len());
        args.iter()
            .map(|arg| match arg {
                MethodCallArg::DataRef(index) => {
                    self.last_args.push(self.input_objects[*index].clone());
                    self.resources[*index].state().data.clone()
                }
                MethodCallArg::MoveValue(value) => {
                    self.last_args.push(ObjectId::Empty);
                    bcs::to_bytes(&value).unwrap()
                }
            })
            .collect()
    }

    pub fn ingest_execution_results(&mut self, execution_result: SerializedReturnValues) {
        self.return_stack.push(execution_result.return_values);

        // map mutable reference outputs to mutations
        for (index, bytes, _) in execution_result.mutable_reference_outputs {
            let object_id = self.last_args.get(index as usize).unwrap();
            match object_id {
                ObjectId::Module(_) => {
                    panic!("module objects cannot be mutated by functions");
                }
                ObjectId::Data(address) => {
                    self.mutations
                        .insert(ObjectId::Data(*address), Op::Modify(bytes));
                }
                _ => {
                    // ignore other object types for mutation purposes
                }
            }
        }
    }

    pub fn mutate(&mut self, object_id: ObjectId, op: Op<Vec<u8>>) {
        self.mutations.insert(object_id, op);
    }

    pub fn modules(&self) -> &ModuleResolver {
        self.session.get_resolver()
    }

    pub fn finalize(mut self) -> VMResult<()> {
        for (module_id, op) in self.session.finish().0?.into_modules().collect::<Vec<_>>() {
            self.mutations.insert(ObjectId::Module(module_id), op);
        }

        for resource in self.resources.iter_mut() {
            if resource.access_metadata().access_type() == AccessType::Write {
                if let Some(op) = self.mutations.remove(&resource.access_metadata().id()) {
                    match op {
                        Op::New(data) | Op::Modify(data) => {
                            resource.state_mut().data = data;
                        }
                        Op::Delete => {
                            panic!("TODO: Handle resource deletion");
                        }
                    }
                }
            }
        }

        if !self.mutations.is_empty() {
            panic!("There are unprocessed mutations: {:?}", self.mutations);
        }

        Ok(())
    }
}
