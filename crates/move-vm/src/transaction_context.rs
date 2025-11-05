use std::collections::HashMap;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::session::Session;
use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, AccessMetadata};
use crate::{ModuleResolver, ObjectId, Transaction};

pub struct TransactionContext<'a> {
    pub resources: &'a mut [AccessHandle<'a, RocksDbStore, Transaction>],
    pub data: Vec<Vec<u8>>,
    pub session: Session<'a, 'a, ModuleResolver>,
}

impl<'a> TransactionContext<'a> {
    pub fn new<'b>(vm: &'b MoveVM, resources: &'a mut [AccessHandle<'a, RocksDbStore, Transaction>]) -> Self where
        'b: 'a,{
        let mut modules = ModuleResolver::new();
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
            session: vm.new_session::<'a>(modules),
        }
    }

    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
    }

    pub fn modules(&self) -> &ModuleResolver {
        self.session.get_resolver()
    }
}