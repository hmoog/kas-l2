use std::{marker::PhantomData, sync::Arc};

use kas_l2_core_atomics::AtomicAsyncLatch;
use kas_l2_runtime_state::{
    StateSpace,
    StateSpace::{Data, LatestPtr, RollbackPtr},
};
use kas_l2_runtime_types::ResourceId;
use kas_l2_storage_manager::concat_bytes;
use kas_l2_storage_types::{Store, WriteStore};

use crate::VmInterface;

/// A rollback operation that reverts state changes from a range of batches.
pub struct Rollback<V: VmInterface> {
    /// The newest batch index to roll back (inclusive).
    from_batch_index: u64,
    /// The oldest batch index to roll back (inclusive).
    to_batch_index: u64,
    /// Latch to signal when rollback is complete.
    done_latch: Arc<AtomicAsyncLatch>,
    /// Marker for the VM interface type (needed for ResourceId).
    _marker: PhantomData<V>,
}

impl<V: VmInterface> Rollback<V> {
    /// Create a new Rollback operation for the specified batch index range.
    /// - `from_batch_index`: The starting batch index (inclusive).
    /// - `to_batch_index`: The ending batch index (inclusive).
    pub fn new(from_batch_index: u64, to_batch_index: u64) -> Self {
        Rollback {
            from_batch_index,
            to_batch_index,
            done_latch: Arc::new(AtomicAsyncLatch::default()),
            _marker: PhantomData,
        }
    }

    /// Get a clone of the done latch to await rollback completion.
    pub fn done_latch(&self) -> Arc<AtomicAsyncLatch> {
        self.done_latch.clone()
    }

    /// Mark the rollback operation as done by opening the latch.
    pub fn done(&self) {
        self.done_latch.open()
    }

    /// Execute the rollback operation on the given store with the provided write batch.
    /// - `store`: The storage backend to operate on.
    /// - `write_batch`: The initial write batch to use.
    /// Returns a new empty write batch after performing the rollback.
    pub fn execute<S: Store<StateSpace = StateSpace>>(
        &self,
        store: &S,
        write_batch: S::WriteBatch,
    ) -> S::WriteBatch {
        store.commit(write_batch);
        store.commit(self.rollback(store, self.from_batch_index, self.to_batch_index));
        store.write_batch()
    }

    /// Perform the rollback operation and return a write batch with the necessary changes.
    /// - `store`: The storage backend to operate on.
    /// - `from`: The starting batch index (inclusive).
    /// - `to`: The ending batch index (inclusive).
    /// Returns a write batch containing the rollback operations.
    fn rollback<S: Store<StateSpace = StateSpace>>(
        &self,
        store: &S,
        from: u64,
        to: u64,
    ) -> S::WriteBatch {
        // Create a new write batch for the rollback operations
        let mut write_batch = store.write_batch();

        // Iterate from most recent batch to oldest (reverse order)
        for batch_index in (to..=from).rev() {
            // Iterate over all rollback pointers for this batch
            for (key, value) in store.prefix_iter(RollbackPtr, &batch_index.to_be_bytes()) {
                // Key format: batch_index (8 bytes) || resource_id
                // Value format: old_version (8 bytes)
                let resource_id = V::ResourceId::from_bytes(&key[8..]);
                let old_version = u64::from_be_bytes(value[..8].try_into().unwrap());

                if let Some(current_version_bytes) = store.get(LatestPtr, &resource_id.to_bytes()) {
                    let current_version =
                        u64::from_be_bytes(current_version_bytes[..8].try_into().unwrap());

                    if old_version == 0 {
                        // Resource did not exist before this batch, so we need to delete it
                        // completely
                        write_batch.delete(LatestPtr, &resource_id.to_bytes());
                    } else {
                        // Delete the data entry for the current version
                        write_batch.delete(
                            Data,
                            &concat_bytes!(&current_version.to_be_bytes(), &resource_id.to_bytes()),
                        );

                        // Restore the latest pointer to the old version
                        write_batch.put(
                            LatestPtr,
                            &resource_id.to_bytes(),
                            &old_version.to_be_bytes(),
                        );
                    }
                }

                // Delete the rollback pointer entry
                write_batch.delete(
                    RollbackPtr,
                    &concat_bytes!(&batch_index.to_be_bytes(), &resource_id.to_bytes()),
                );
            }
        }

        // Return the constructed write batch
        write_batch
    }
}
