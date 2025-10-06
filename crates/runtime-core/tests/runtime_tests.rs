extern crate core;

use std::{collections::HashMap, thread::sleep, time::Duration};

use kas_l2_runtime_core::{AccessHandle, Batch, RuntimeBuilder};

#[test]
pub fn test_runtime() {
    let mut runtime = RuntimeBuilder::default()
        .with_execution_workers(4)
        .with_storage(KVStore(HashMap::new()))
        .with_transaction_processor(
            |tx: &Transaction, _resources: &mut [AccessHandle<Transaction>]| {
                eprintln!("Processed transaction with id {}", tx.0);
                Ok::<(), ()>(())
            },
        )
        .with_batch_processor(|batch: Batch<Transaction>| {
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

pub struct KVStore(HashMap<u32, Vec<u8>>);

struct Transaction(u32, Vec<Access>);

#[derive(Clone)]
pub enum Access {
    Read(usize),
    Write(usize),
}

mod runtime_traits {
    use kas_l2_runtime_core::AccessType;

    use crate::{Access, KVStore, Transaction};

    impl kas_l2_runtime_core::Storage<u32> for KVStore {
        type Error = std::io::Error;

        fn get(&self, key: &u32) -> Result<Option<Vec<u8>>, Self::Error> {
            Ok(self.0.get(key).cloned())
        }

        fn put(&mut self, key: u32, value: Vec<u8>) -> Result<(), Self::Error> {
            self.0.insert(key, value);
            Ok(())
        }

        fn delete(&mut self, key: &u32) -> Result<bool, Self::Error> {
            Ok(self.0.remove(key).is_some())
        }
    }

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
