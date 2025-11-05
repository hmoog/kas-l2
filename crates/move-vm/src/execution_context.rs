use std::collections::HashMap;
use move_binary_format::errors::VMResult;
use move_core_types::effects::Op;
use move_core_types::runtime_value::MoveTypeLayout;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::session::{SerializedReturnValues, Session};
use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, AccessMetadata, AccessType};
use crate::{ModuleResolver, ObjectId, Transaction};
use crate::instructions::function_call::FunctionCallArg;

pub struct ExecutionContext<'a, 'v, 'res> {
    pub resources: &'res mut [AccessHandle<'a, RocksDbStore, Transaction>],
    pub data: Vec<Vec<u8>>,
    pub mutations: HashMap<ObjectId, Op<Vec<u8>>>,
    pub return_stack: Vec<Vec<(Vec<u8>, MoveTypeLayout)>>,
    pub session: Session<'a, 'v, ModuleResolver>,
}

impl<'elem, 'vm, 'res> ExecutionContext<'elem, 'vm, 'res> {
    pub fn new(vm: &'vm MoveVM, resources: &'res mut [AccessHandle<'elem, RocksDbStore, Transaction>]) -> Self {
        let mut modules = ModuleResolver::default();
        let mut data = vec![];

        for resource in resources.iter() {
            match resource.access_metadata().id() {
                ObjectId::Data(_) => data.push(resource.state().data.clone()),
                ObjectId::Module(module_id) => {
                    if !resource.is_new() {
                        modules.add_module(module_id.clone(), resource.state().data.clone())
                    }
                }
            }
        }

        Self {
            resources,
            data,
            return_stack: Vec::new(),
            mutations: HashMap::new(),
            session: vm.new_session(modules),
        }
    }

    pub fn serialized_args(&self, args: &Vec<FunctionCallArg>) -> Vec<Vec<u8>> {
        args.iter()
            .map(|arg| match arg {
                FunctionCallArg::DataRef(index) => self.data[*index].clone(),
                FunctionCallArg::MoveValue(value) => bcs::to_bytes(&value).unwrap(),
            })
            .collect()
    }

    pub fn ingest_execution_results(&mut self, execution_result: SerializedReturnValues) {
        self.return_stack.push(execution_result.return_values);

        for (_index, _bytes, _type_layout) in execution_result.mutable_reference_outputs {
            // map index to object id
            // store bytes as update mutations to the corresponding object id
        }
    }

    pub fn mutate(&mut self, object_id: ObjectId, op: Op<Vec<u8>>) {
        self.mutations.insert(object_id, op);
    }

    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
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