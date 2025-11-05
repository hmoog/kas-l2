extern crate core;

use std::{str::FromStr, sync::Arc, thread::sleep, time::Duration};

use kas_l2_move_utils::CompiledModules;
use kas_l2_move_vm::{Instruction, ObjectAccess::Write, ObjectId, Transaction, VM};
use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{Batch, RuntimeBuilder};
use kas_l2_storage::StorageConfig;
use move_core_types::{account_address::AccountAddress};
use tempfile::TempDir;
use kas_l2_move_vm::instructions::PublishModules;

#[test]
pub fn test_runtime() -> Result<(), anyhow::Error> {
    let modules = CompiledModules::from_sources(
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
    );

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let store = RocksDbStore::open(temp_dir.path());
        let move_vm = Arc::new(VM::new());

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

        runtime.process(vec![Transaction {
            accessed_resources: vec![Write(ObjectId::from_str("0x1::Test")?)],
            instruction: Instruction::PublishModules(PublishModules{
                modules: modules.serialize(vec!["0x1::Test"])?,
                sender: AccountAddress::from_str("0x1")?,
            }),
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

        Ok(())
    }
}
