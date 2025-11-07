extern crate core;

use std::{thread::sleep, time::Duration};

use kas_l2_runtime_builder::RuntimeBuilder;
use kas_l2_runtime_core::Batch;
use kas_l2_storage_manager::StorageConfig;
use kas_l2_storage_rocksdb_store::RocksDbStore;
use tempfile::TempDir;

use crate::test_framework::{Access, AssertWrittenState, Tx};

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let store = RocksDbStore::open(temp_dir.path());

        let mut runtime = RuntimeBuilder::default()
            .with_storage_config(StorageConfig::default().with_store(store.clone()))
            .with_transaction_processor(Tx::process)
            .with_notarization(|batch: &Batch<RocksDbStore, Tx>| {
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

        for assertion in [
            AssertWrittenState(1, vec![0, 1, 3]),
            AssertWrittenState(2, vec![1]),
            AssertWrittenState(3, vec![]),
            AssertWrittenState(10, vec![4]),
            AssertWrittenState(20, vec![4]),
        ] {
            assertion.assert(&store);
        }

        runtime.shutdown();
    }
}

mod test_framework {
    use kas_l2_runtime_core::{
        AccessHandle, AccessMetadata, AccessType, RuntimeState, Transaction, VersionedState,
    };
    use kas_l2_storage_manager::ReadStore;
    use kas_l2_storage_rocksdb_store::RocksDbStore;

    pub struct Tx(pub usize, pub Vec<Access>);

    impl Tx {
        pub fn process(&self, resources: &mut [AccessHandle<RocksDbStore, Tx>]) -> Result<(), ()> {
            for resource in resources {
                if resource.access_metadata().access_type() == AccessType::Write {
                    resource
                        .state_mut()
                        .data
                        .extend_from_slice(&self.0.to_be_bytes());
                }
            }
            Ok::<(), ()>(())
        }
    }

    impl Transaction for Tx {
        type ResourceId = usize;
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

    impl AccessMetadata<usize> for Access {
        fn id(&self) -> usize {
            match self {
                Access::Read(id) => *id,
                Access::Write(id) => *id,
            }
        }

        fn access_type(&self) -> AccessType {
            match self {
                Access::Read(_) => AccessType::Read,
                Access::Write(_) => AccessType::Write,
            }
        }
    }

    pub struct AssertWrittenState(pub usize, pub Vec<usize>);

    impl AssertWrittenState {
        pub fn assert<S: ReadStore<StateSpace = RuntimeState>>(&self, store: &S) {
            let writer_count = self.1.len();
            let writer_log: Vec<u8> = self.1.iter().flat_map(|id| id.to_be_bytes()).collect();

            let versioned_state = VersionedState::<Tx>::from_latest_data(store, self.0);
            assert_eq!(versioned_state.version(), writer_count as u64);
            assert_eq!(versioned_state.state().data, writer_log);
        }
    }
}
