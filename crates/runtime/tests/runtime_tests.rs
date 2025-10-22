extern crate core;

use std::{thread::sleep, time::Duration};

use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{AccessHandle, Batch, RuntimeBuilder};
use kas_l2_storage::StorageConfig;
use tempfile::TempDir;

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    {
        let store = RocksDbStore::open(temp_dir.path());

        let mut runtime = RuntimeBuilder::default()
            .with_execution_workers(4)
            .with_storage_config(StorageConfig::default().with_store(store))
            .with_transaction_processor(
                |tx: &Transaction, _resources: &mut [AccessHandle<RocksDbStore, Transaction>]| {
                    eprintln!("Processed transaction with id {}", tx.0);
                    Ok::<(), ()>(())
                },
            )
            .with_batch_processor(|batch: Batch<RocksDbStore, Transaction>| {
                eprintln!(
                    ">> Processed batch with {} transactions and {} state changes",
                    batch.txs().len(),
                    batch.state_diffs().len()
                );
            })
            .build();

        runtime.process(vec![
            Transaction(0, vec![Access::Write(1), Access::Read(3)]),
            Transaction(1, vec![Access::Write(1), Access::Write(2)]),
            Transaction(2, vec![Access::Read(3)]),
        ]);

        runtime.process(vec![
            Transaction(3, vec![Access::Write(1), Access::Read(3)]),
            Transaction(4, vec![Access::Write(10), Access::Write(20)]),
        ]);

        sleep(Duration::from_secs(1));

        runtime.shutdown();
    }
}

struct Transaction(u32, Vec<Access>);

#[derive(Clone)]
pub enum Access {
    Read(usize),
    Write(usize),
}

mod runtime_traits {
    use kas_l2_runtime_core::AccessType;

    use crate::{Access, Transaction};

    impl kas_l2_runtime_core::Transaction for Transaction {
        type ResourceId = u32;
        type AccessMetadata = Access;
        fn accessed_resources(&self) -> &[Self::AccessMetadata] {
            &self.1
        }
    }

    impl kas_l2_runtime_core::AccessMetadata<u32> for Access {
        fn id(&self) -> u32 {
            match self {
                Access::Read(id) => *id as u32,
                Access::Write(id) => *id as u32,
            }
        }

        fn access_type(&self) -> AccessType {
            match self {
                Access::Read(_) => AccessType::Read,
                Access::Write(_) => AccessType::Write,
            }
        }
    }
}
