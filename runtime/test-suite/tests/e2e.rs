extern crate core;

use kas_l2_runtime_manager::{ExecutionConfig, RuntimeManager};
use kas_l2_runtime_rocksdb_store::RocksDbStore;
use kas_l2_storage_manager::StorageConfig;
use tempfile::TempDir;

use crate::test_framework::{Access, AssertResourceDeleted, AssertWrittenState, TestVM, Tx};

#[test]
pub fn test_runtime() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let storage: RocksDbStore = RocksDbStore::open(temp_dir.path());
        let mut runtime = RuntimeManager::new(
            ExecutionConfig::default().with_vm(TestVM),
            StorageConfig::default().with_store(storage),
        );

        let batch1 = runtime.schedule(vec![
            Tx(0, vec![Access::Write(1), Access::Read(3)]),
            Tx(1, vec![Access::Write(1), Access::Write(2)]),
            Tx(2, vec![Access::Read(3)]),
        ]);

        let batch2 = runtime.schedule(vec![
            Tx(3, vec![Access::Write(1), Access::Read(3)]),
            Tx(4, vec![Access::Write(10), Access::Write(20)]),
        ]);

        batch1.wait_committed_blocking();
        batch2.wait_committed_blocking();

        for assertion in [
            AssertWrittenState(1, vec![0, 1, 3]),
            AssertWrittenState(2, vec![1]),
            AssertWrittenState(3, vec![]),
            AssertWrittenState(10, vec![4]),
            AssertWrittenState(20, vec![4]),
        ] {
            assertion.assert(runtime.storage_manager().store());
        }

        runtime.shutdown();
    }
}

#[test]
pub fn test_rollback() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    {
        let storage: RocksDbStore = RocksDbStore::open(temp_dir.path());
        let mut runtime = RuntimeManager::new(
            ExecutionConfig::default().with_vm(TestVM),
            StorageConfig::default().with_store(storage),
        );

        // Batch 0: Write to resources 1 and 2
        let batch0 =
            runtime.schedule(vec![Tx(0, vec![Access::Write(1)]), Tx(1, vec![Access::Write(2)])]);
        batch0.wait_committed_blocking();

        // Batch 1: Write to resources 1 and 3
        let batch1 =
            runtime.schedule(vec![Tx(2, vec![Access::Write(1)]), Tx(3, vec![Access::Write(3)])]);
        batch1.wait_committed_blocking();

        // Batch 2: Write to resources 1 and 4
        let batch2 =
            runtime.schedule(vec![Tx(4, vec![Access::Write(1)]), Tx(5, vec![Access::Write(4)])]);
        batch2.wait_committed_blocking();

        // Verify state before rollback
        for assertion in [
            AssertWrittenState(1, vec![0, 2, 4]), // Written by tx 0, 2, 4
            AssertWrittenState(2, vec![1]),       // Written by tx 1
            AssertWrittenState(3, vec![3]),       // Written by tx 3
            AssertWrittenState(4, vec![5]),       // Written by tx 5
        ] {
            assertion.assert(runtime.storage_manager().store());
        }

        // Rollback to index 1 (revert batches with index 1 and 2, keep batch with index 0)
        runtime.rollback_to(1);

        // Verify state after rollback - only batch0 effects should remain
        for assertion in [
            AssertWrittenState(1, vec![0]), // Only tx 0's write remains
            AssertWrittenState(2, vec![1]), // tx 1's write remains (in batch0)
        ] {
            assertion.assert(runtime.storage_manager().store());
        }

        // Resources 3 and 4 should no longer exist (created in rolled-back batches)
        AssertResourceDeleted(3).assert(runtime.storage_manager().store());
        AssertResourceDeleted(4).assert(runtime.storage_manager().store());

        runtime.shutdown();
    }
}

mod test_framework {
    use kas_l2_runtime_manager::{AccessHandle, RuntimeBatch, VmInterface};
    use kas_l2_runtime_state::{StateSpace, VersionedState};
    use kas_l2_runtime_types::{AccessMetadata, AccessType, Transaction};
    use kas_l2_storage_types::{ReadStore, Store};

    #[derive(Clone)]
    pub struct TestVM;

    impl VmInterface for TestVM {
        type Transaction = Tx;
        type TransactionEffects = ();
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
        fn accessed_resources(&self) -> &[<TestVM as VmInterface>::AccessMetadata] {
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

    pub struct AssertResourceDeleted(pub usize);

    impl AssertResourceDeleted {
        pub fn assert<S: ReadStore<StateSpace = StateSpace>>(&self, store: &S) {
            let id_bytes = self.0.to_be_bytes();
            assert!(
                store.get(StateSpace::LatestPtr, &id_bytes).is_none(),
                "Resource {} should have been deleted but still exists",
                self.0
            );
        }
    }
}
