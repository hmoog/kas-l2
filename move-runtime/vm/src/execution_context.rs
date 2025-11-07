use std::collections::HashMap;

use kas_l2_runtime_core::{AccessHandle, AccessMetadata, AccessType, RuntimeState};
use kas_l2_storage_manager::Store;
use move_binary_format::errors::VMResult;
use move_core_types::{effects::Op, runtime_value::MoveTypeLayout};
use move_vm_runtime::{
    move_vm::MoveVM,
    session::{SerializedReturnValues, Session},
};

use crate::{MethodCallArg, Modules, ObjectId, VM};

pub struct ExecutionContext<'a, 'v, 'r, S: Store<StateSpace = RuntimeState>> {
    pub resources: &'r mut [AccessHandle<'a, S, VM>],
    pub input_objects: Vec<ObjectId>,
    pub mutations: HashMap<ObjectId, Op<Vec<u8>>>,
    pub last_args: Vec<ObjectId>,
    pub return_stack: Vec<Vec<(Vec<u8>, MoveTypeLayout)>>,
    pub session: Session<'a, 'v, Modules>,
}

impl<'a, 'v, 'r, S: Store<StateSpace = RuntimeState>> ExecutionContext<'a, 'v, 'r, S> {
    pub fn new(vm: &'v MoveVM, resources: &'r mut [AccessHandle<'a, S, VM>]) -> Self {
        let mut modules = Modules::default();
        let mut input_objects = Vec::with_capacity(resources.len());

        for resource in resources.iter() {
            let object_id = resource.access_metadata().id();

            if let ObjectId::Module(module_id) = &object_id {
                if !resource.is_new() {
                    modules.add(module_id.clone(), resource.state().data.clone())
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

    pub fn prepare_args(&mut self, args: &[MethodCallArg]) -> Vec<Vec<u8>> {
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

        // map mutable reference changes to mutations
        for (index, bytes, _) in execution_result.mutable_reference_outputs {
            let object_id = self.last_args.get(index as usize).unwrap();
            match object_id {
                ObjectId::Module(_) => {
                    panic!("module objects cannot be mutated by functions");
                }
                ObjectId::Data(address) => {
                    self.mutations.insert(ObjectId::Data(*address), Op::Modify(bytes));
                }
                _ => {
                    // ignore other object types for mutation purposes
                }
            }
        }
    }

    pub fn modules(&self) -> &Modules {
        self.session.get_resolver()
    }

    pub fn finalize(mut self) -> VMResult<()> {
        // collect module mutations
        for (module_id, op) in self.session.finish().0?.into_modules() {
            self.mutations.insert(ObjectId::Module(module_id), op);
        }

        // apply mutations back to resources
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

        // ensure there are no pending mutations
        if !self.mutations.is_empty() {
            panic!("There are unprocessed mutations: {:?}", self.mutations);
        }

        Ok(())
    }
}
