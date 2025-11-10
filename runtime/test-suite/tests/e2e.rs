extern crate core;

use std::{thread::sleep, time::Duration};

use kas_l2_runtime_execution_dag::{ExecutionConfig, ExecutionDag};
use kas_l2_runtime_storage_manager::StorageConfig;
use kas_l2_storage_rocksdb_store::RocksDbStore;
use tempfile::TempDir;

use crate::test_framework::{Access, AssertWrittenState, TestVM, Tx};

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let store: RocksDbStore = RocksDbStore::open(temp_dir.path());

        let mut runtime = ExecutionDag::new(
            ExecutionConfig::default().with_vm(TestVM),
            StorageConfig::default().with_store(store.clone()),
        );

        runtime.schedule(vec![
            Tx(0, vec![Access::Write(1), Access::Read(3)]),
            Tx(1, vec![Access::Write(1), Access::Write(2)]),
            Tx(2, vec![Access::Read(3)]),
        ]);

        runtime.schedule(vec![
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
    use kas_l2_runtime_execution_dag::{AccessHandle, RuntimeBatch, VM};
    use kas_l2_runtime_interface::{AccessMetadata, AccessType, Transaction};
    use kas_l2_runtime_state::VersionedState;
    use kas_l2_runtime_state_space::StateSpace;
    use kas_l2_storage_interface::{ReadStore, Store};

    #[derive(Clone)]
    pub struct TestVM;

    impl VM for TestVM {
        type Transaction = Tx;
        type ResourceId = usize;
        type Ownership = usize;
        type AccessMetadata = Access;
        type Error = ();

        fn process_transaction<S: Store<StateSpace = StateSpace>>(
            &self,
            tx: &Self::Transaction,
            resources: &mut [AccessHandle<S, Self>],
        ) -> Result<(), Self::Error> {
            for resource in resources {
                if resource.access_metadata().access_type() == AccessType::Write {
                    resource.state_mut().data.extend_from_slice(&tx.0.to_be_bytes());
                }
            }
            Ok::<(), ()>(())
        }

        fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, batch: &RuntimeBatch<S, Self>) {
            eprintln!(
                ">> Processed batch with {} transactions and {} state changes",
                batch.txs().len(),
                batch.state_diffs().len()
            );
        }
    }

    pub struct Tx(pub usize, pub Vec<Access>);

    impl Transaction<usize, Access> for Tx {
        fn accessed_resources(&self) -> &[<TestVM as VM>::AccessMetadata] {
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
        pub fn assert<S: ReadStore<StateSpace = StateSpace>>(&self, store: &S) {
            let writer_count = self.1.len();
            let writer_log: Vec<u8> = self.1.iter().flat_map(|id| id.to_be_bytes()).collect();

            let versioned_state = VersionedState::<usize, usize>::from_latest_data(store, self.0);
            assert_eq!(versioned_state.version(), writer_count as u64);
            assert_eq!(versioned_state.state().data, writer_log);
        }
    }
}
