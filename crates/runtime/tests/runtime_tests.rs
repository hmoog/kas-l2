extern crate core;

use std::{thread::sleep, time::Duration};

use kas_l2_rocksdb_store::RocksDbStore;
use kas_l2_runtime_core::{
    AccessHandle, AccessMetadata, AccessType, Batch, RuntimeBuilder, Transaction, VersionedState,
};
use kas_l2_storage::StorageConfig;
use tempfile::TempDir;

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    {
        let store = RocksDbStore::open(temp_dir.path());

        let mut runtime = RuntimeBuilder::default()
            .with_storage_config(StorageConfig::default().with_store(store.clone()))
            .with_transaction_processor(Tx::process)
            .with_batch_processor(|batch: Batch<RocksDbStore, Tx>| {
                eprintln!(
                    ">> Processed batch with {} transactions and {} state changes",
                    batch.txs().len(),
                    batch.state_diffs().len()
                );
            })
            .build();

        runtime.process(vec![
            Tx(0, vec![Access::Write(1), Access::Read(3)]),
            Tx(1, vec![Access::Write(1), Access::Write(2)]),
            Tx(2, vec![Access::Read(3)]),
        ]);

        runtime.process(vec![
            Tx(3, vec![Access::Write(1), Access::Read(3)]),
            Tx(4, vec![Access::Write(10), Access::Write(20)]),
        ]);

        sleep(Duration::from_secs(1));

        assert_eq!(
            3,
            vec_to_u64(&VersionedState::<Tx>::from_store(&store, 1u32).state.data)
        );

        runtime.shutdown();
    }
}

struct Tx(u32, Vec<Access>);

impl Tx {
    pub fn process(&self, _resources: &mut [AccessHandle<RocksDbStore, Tx>]) -> Result<(), ()> {
        for resource in _resources {
            if resource.access_metadata().access_type() == AccessType::Write {
                let resource_id = resource.access_metadata().id();
                let state = &mut resource.state_mut();

                let read_value = vec_to_u64(&state.data);
                let written_value = read_value.wrapping_add(1);
                eprintln!(
                    "tx{}.resource[{}] = {}; // prev: {}",
                    self.0, resource_id, written_value, read_value
                );

                state.data = u64_to_vec(written_value);
            }
        }
        Ok::<(), ()>(())
    }
}

impl Transaction for Tx {
    type ResourceId = u32;
    type AccessMetadata = Access;

    fn accessed_resources(&self) -> &[Self::AccessMetadata] {
        &self.1
    }
}

#[derive(Clone)]
pub enum Access {
    Read(usize),
    Write(usize),
}

impl AccessMetadata<u32> for Access {
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

fn vec_to_u64(bytes: &Vec<u8>) -> u64 {
    if bytes.len() == 8 {
        u64::from_le_bytes(bytes.as_slice().try_into().unwrap())
    } else {
        0
    }
}

fn u64_to_vec(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
