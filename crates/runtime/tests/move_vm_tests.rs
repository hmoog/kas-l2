extern crate core;

use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};
use std::str::FromStr;
use kas_l2_move_utils::Compiler;
use kas_l2_move_vm::{Instruction, ObjectAccess::Write, ObjectId, Transaction, VM};
use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{Batch, RuntimeBuilder};
use kas_l2_storage::StorageConfig;
use move_core_types::{
    account_address::AccountAddress, language_storage::ModuleId,
};
use tempfile::TempDir;

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let store = RocksDbStore::open(temp_dir.path());
        let move_vm = Arc::new(VM::new());

        let mut compiled_modules = HashMap::new();
        for module in Compiler::compile_sources(
            &[r#"
            module 0x1::Test {
                // Define a resource type that we can pass around
                public struct Obj has key { value: u64 }

                public fun f(o: &mut Obj) {
                    o.value = o.value + 1;
                }
            }
            "#],
            &[],
        ) {
            compiled_modules.insert(
                module.module_id().1,
                module.into_compiled_unit(),
            );
        }

        let mut runtime = RuntimeBuilder::default()
            .with_storage_config(StorageConfig::default().with_store(store.clone()))
            .with_transaction_processor(move |tx, resource_handles| {
                move_vm.process_transaction(tx, resource_handles)
            })
            .with_notarization(|batch: &Batch<RocksDbStore, Transaction>| {
                eprintln!(
                    ">> Processed batch with {} transactions and {} state changes",
                    batch.txs().len(),
                    batch.state_diffs().len()
                );
            })
            .build();

        let module1_id = ModuleId::from_str("0x1::Test").unwrap();

        runtime.process(vec![Transaction {
            accessed_resources: vec![Write(ObjectId::Module(module1_id.clone()))],
            instruction: Instruction::PublishModules {
                modules: vec![compiled_modules.get(&module1_id).unwrap().serialize()],
                sender: AccountAddress::ONE,
            },
        }]);

        sleep(Duration::from_secs(1));

        // for assertion in [
        //     AssertWrittenState(1, vec![0, 1, 3]),
        //     AssertWrittenState(2, vec![1]),
        //     AssertWrittenState(3, vec![]),
        //     AssertWrittenState(10, vec![4]),
        //     AssertWrittenState(20, vec![4]),
        // ] {
        //     assertion.assert(&store);
        // }

        runtime.shutdown();
    }
}
