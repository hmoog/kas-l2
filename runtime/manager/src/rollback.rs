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

/// Represents a rollback operation that reverts state changes made within a specified range of
/// batch indices.
///
/// # Type Parameters
///
/// * `V`: The VM interface type that defines the resource ID type.
pub struct Rollback<V: VmInterface> {
    /// The lower bound of the batch index range to rollback (inclusive).
    lower_bound: u64,
    /// The upper bound of the batch index range to rollback (inclusive).
    upper_bound: u64,
    /// Latch to signal when the rollback operation is complete.
    done_latch: Arc<AtomicAsyncLatch>,
    /// Marker for the VM interface type.
    _marker: PhantomData<V>,
}

impl<V: VmInterface> Rollback<V> {
    /// Create a new rollback operation for the specified batch index range.
    ///
    /// # Arguments
    ///
    /// * `lower_bound`: The starting batch index (inclusive) for the rollback.
    /// * `upper_bound`: The ending batch index (inclusive) for the rollback.
    ///
    /// # Returns
    ///
    /// A new `Rollback` instance.
    pub fn new(lower_bound: u64, upper_bound: u64) -> Self {
        Rollback {
            lower_bound,
            upper_bound,
            done_latch: Arc::new(AtomicAsyncLatch::default()),
            _marker: PhantomData,
        }
    }

    /// Get a clone of the done latch to await rollback completion.
    ///
    /// # Returns
    ///
    /// An `Arc` to the `AtomicAsyncLatch` that signals when the rollback is done.
    pub fn done_latch(&self) -> Arc<AtomicAsyncLatch> {
        self.done_latch.clone()
    }

    /// Signal that the rollback operation is complete.
    pub fn done(&self) {
        self.done_latch.open()
    }

    /// Execute the rollback operation on the given store, committing the provided write batch
    /// before applying the rollback.
    ///
    /// # Type Parameters
    ///
    /// * `S`: The storage backend type.
    ///
    /// # Arguments
    ///
    /// * `store`: The storage backend to operate on.
    /// * `write_batch`: An existing write batch that will be committed before rolling back.
    ///
    /// # Returns
    ///
    /// A new empty write batch, allowing the caller to continue issuing write operations after the
    /// rollback.
    pub fn execute<S: Store<StateSpace = StateSpace>>(
        &self,
        store: &S,
        write_batch: S::WriteBatch,
    ) -> S::WriteBatch {
        // Commit any existing changes so the rollback sees a consistent state.
        store.commit(write_batch);

        // Perform the rollback and commit the write batch with the changes.
        store.commit(self.build_rollback_batch(store));

        // Return a new empty write batch for further operations.
        store.write_batch()
    }

    /// Generate a write batch containing the necessary rollback operations.
    ///
    /// # Type Parameters
    ///
    /// * `S`: The storage backend type.
    ///
    /// # Arguments
    ///
    /// * `store`: The storage backend to operate on.
    ///
    /// # Returns
    ///
    /// A write batch containing the rollback operations.
    fn build_rollback_batch<S: Store<StateSpace = StateSpace>>(&self, store: &S) -> S::WriteBatch {
        // Create a new write batch for the rollback operations.
        let mut write_batch = store.write_batch();

        // Iterate from most recent batch to oldest (reverse order).
        for index in (self.lower_bound..=self.upper_bound).rev() {
            // Iterate over all rollback pointers for this batch.
            for (key, value) in store.prefix_iter(RollbackPtr, &index.to_be_bytes()) {
                // Parse resource id from key (format: batch_index (8 bytes) || resource_id).
                let resource_id = V::ResourceId::from_bytes(&key[8..]);

                // Parse old version from value (format: old version (8 bytes)).
                let old_version = u64::from_be_bytes(value[..8].try_into().unwrap());

                // Apply rollback pointer to write batch.
                self.apply_rollback_ptr(store, &mut write_batch, index, resource_id, old_version);
            }
        }

        // Return the constructed write batch.
        write_batch
    }

    /// Process a single rollback pointer, updating the write batch accordingly.
    ///
    /// # Type Parameters
    ///
    /// * `S`: The storage backend type.
    ///
    /// # Arguments
    ///
    /// * `store`: The storage backend to operate on.
    /// * `write_batch`: The write batch to update with rollback operations.
    /// * `batch_index`: The batch index associated with the rollback pointer.
    /// * `resource_id`: The resource ID associated with the rollback pointer.
    /// * `old_version`: The old_version to restore for the resource.
    fn apply_rollback_ptr<S: Store<StateSpace = StateSpace>>(
        &self,
        store: &S,
        write_batch: &mut S::WriteBatch,
        batch_index: u64,
        resource_id: V::ResourceId,
        old_version: u64,
    ) {
        // Check if there is a latest pointer for this resource.
        if let Some(current_version_bytes) = store.get(LatestPtr, &resource_id.to_bytes()) {
            // Get the current version from the latest pointer.
            let current_version =
                u64::from_be_bytes(current_version_bytes[..8].try_into().unwrap());

            // Delete the data entry for the current version.
            write_batch.delete(
                Data,
                &concat_bytes!(&current_version.to_be_bytes(), &resource_id.to_bytes()),
            );
        }

        if old_version == 0 {
            // Resource did not exist before this batch, so we need to delete it.
            write_batch.delete(LatestPtr, &resource_id.to_bytes());
        } else {
            // Restore the latest pointer to the old version.
            write_batch.put(LatestPtr, &resource_id.to_bytes(), &old_version.to_be_bytes());
        }

        // Delete the rollback pointer entry.
        write_batch.delete(
            RollbackPtr,
            &concat_bytes!(&batch_index.to_be_bytes(), &resource_id.to_bytes()),
        );
    }
}
