extern crate core;

use std::{str::FromStr, sync::Arc, thread::sleep, time::Duration};

use kas_l2_move_runtime_utils::CompiledModules;
use kas_l2_move_runtime_vm::{
    Instruction,
    ObjectAccess::{Read, Write},
    ObjectId, Transaction, VM,
};
use kas_l2_runtime::{Batch, RuntimeBuilder};
use kas_l2_storage_manager::StorageConfig;
use kas_l2_storage_rocksdb_store::RocksDbStore;
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use tempfile::TempDir;

#[test]
pub fn test_runtime() -> Result<(), anyhow::Error> {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let store = RocksDbStore::open(temp_dir.path());
        let vm = Arc::new(VM::default());

        let mut runtime = RuntimeBuilder::default()
            .with_storage_config(StorageConfig::default().with_store(store.clone()))
            .with_transaction_processor(move |tx, res| {
                vm.process(tx, res).map_err(|err| {
                    eprintln!("tx execution failed: {:?}", err);
                    err
                })
            })
            .with_notarization(|batch: &Batch<RocksDbStore, Transaction>| {
                eprintln!(
                    ">> Processed batch with {} transactions and {} state changes",
                    batch.txs().len(),
                    batch.state_diffs().len()
                );
            })
            .build();

        runtime.process(vec![
            Transaction {
                accessed_resources: vec![Write(ObjectId::from_str("0x1::Test")?)],
                instruction: Instruction::PublishModules {
                    modules: test_modules().serialize(vec!["0x1::Test"])?,
                    sender: AccountAddress::from_str("0x1")?,
                },
            },
            Transaction {
                accessed_resources: vec![Read(ObjectId::from_str("0x1::Test")?)],
                instruction: Instruction::MethodCall {
                    module_ref: 0,
                    function_name: Identifier::from_str("get_value")?,
                    ty_args: vec![],
                    args: vec![],
                },
            },
        ]);

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

        Ok(())
    }
}

fn test_modules() -> CompiledModules {
    CompiledModules::from_sources(
        &[r#"
            module 0x1::Test {
                // Define a resource type that we can pass around
                public struct Obj has key { value: u64 }

                public fun new_obj(): Obj {
                    Obj { value: 0 }
                }

                public fun f(o: &mut Obj) {
                    o.value = o.value + 1;
                }

                public fun get_value(): u64 {
                    7
                }
            }
            "#],
        &[],
    )
}
