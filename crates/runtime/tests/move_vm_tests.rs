extern crate core;

use std::{thread::sleep, time::Duration};

use kas_l2_move_vm::{
    Instruction,
    ObjectAccess::{Read, Write},
    ObjectId::Module,
    Transaction, VM,
};
use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{Batch, RuntimeBuilder};
use kas_l2_storage::StorageConfig;
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use tempfile::TempDir;

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let store = RocksDbStore::open(temp_dir.path());
        let move_vm = VM::new();

        let mut runtime = RuntimeBuilder::default()
            .with_storage_config(StorageConfig::default().with_store(store.clone()))
            .with_transaction_processor(|tx, resource_handles| {
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
            accessed_resources: vec![Write(Module(ModuleId::new(
                AccountAddress::ONE,
                "test".into(),
            )))],
            instruction: Instruction::PublishModules {
                modules: vec![],
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
