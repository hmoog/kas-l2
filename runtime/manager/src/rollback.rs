use std::{marker::PhantomData, sync::Arc};

use vprogs_core_atomics::AtomicAsyncLatch;
use vprogs_runtime_state::{
    StateSpace,
    StateSpace::{Data, LatestPtr, RollbackPtr},
};
use vprogs_runtime_types::ResourceId;
use vprogs_storage_manager::concat_bytes;
use vprogs_storage_types::{Store, WriteBatch};

use crate::VmInterface;

/// Represents a rollback operation that reverts state changes made within an inclusive range of
/// batch indices.
///
/// A rollback walks batches in reverse order and restores each affected resource to the version it
/// had before the batch was applied.
pub struct Rollback<V: VmInterface> {
    /// Lower bound of the batch index range to roll back (inclusive).
    lower_bound: u64,
    /// Upper bound of the batch index range to roll back (inclusive).
    upper_bound: u64,
    /// Signal that resolves when the rollback operation is complete.
    done_signal: Arc<AtomicAsyncLatch>,
    /// Marker for the VM interface type.
    _marker: PhantomData<V>,
}

impl<V: VmInterface> Rollback<V> {
    /// Creates a new rollback operation for the given inclusive batch range.
    pub fn new(lower_bound: u64, upper_bound: u64, done_signal: &Arc<AtomicAsyncLatch>) -> Self {
        Rollback {
            lower_bound,
            upper_bound,
            done_signal: done_signal.clone(),
            _marker: PhantomData,
        }
    }

    /// Executes the rollback on `store`.
    ///
    /// Any pending writes in `write_batch` are committed first so that the rollback operates on a
    /// consistent view of state. The rollback changes are then applied and committed, and a fresh
    /// write batch is returned for further writes.
    pub fn execute<S: Store<StateSpace = StateSpace>>(
        &self,
        store: &S,
        write_batch: S::WriteBatch,
    ) -> S::WriteBatch {
        // Commit any existing changes so the rollback sees a consistent state.
        store.commit(write_batch);

        // Perform the rollback and commit the resulting changes.
        store.commit(self.build_rollback_batch(store));

        // Return a new empty write batch for further operations.
        store.write_batch()
    }

    /// Signals that the rollback operation has completed.
    pub fn done(&self) {
        self.done_signal.open()
    }

    /// Builds a write batch containing all rollback operations.
    fn build_rollback_batch<S: Store<StateSpace = StateSpace>>(&self, store: &S) -> S::WriteBatch {
        let mut write_batch = store.write_batch();

        // Walk batches from newest to oldest.
        for index in (self.lower_bound..=self.upper_bound).rev() {
            // Apply all rollback pointers associated with this batch.
            for (key, value) in store.prefix_iter(RollbackPtr, &index.to_be_bytes()) {
                // Key layout: batch_index (8 bytes) || resource_id
                let resource_id = V::ResourceId::from_bytes(&key[8..]);

                // Value layout: old_version (8 bytes)
                let old_version = u64::from_be_bytes(value[..8].try_into().unwrap());

                self.apply_rollback_ptr(store, &mut write_batch, index, resource_id, old_version);
            }
        }

        write_batch
    }

    /// Applies a single rollback pointer to the write batch.
    ///
    /// This removes the current version of the resource (if any), restores the previous version,
    /// and deletes the rollback pointer entry.
    fn apply_rollback_ptr<S: Store<StateSpace = StateSpace>>(
        &self,
        store: &S,
        write_batch: &mut S::WriteBatch,
        batch_index: u64,
        resource_id: V::ResourceId,
        old_version: u64,
    ) {
        // Remove the currently live version, if present.
        if let Some(current_version_bytes) = store.get(LatestPtr, &resource_id.to_bytes()) {
            let current_version =
                u64::from_be_bytes(current_version_bytes[..8].try_into().unwrap());

            write_batch.delete(
                Data,
                &concat_bytes!(&current_version.to_be_bytes(), &resource_id.to_bytes()),
            );
        }

        if old_version == 0 {
            // The resource did not exist before this batch.
            write_batch.delete(LatestPtr, &resource_id.to_bytes());
        } else {
            // Restore the resource to its previous version.
            write_batch.put(LatestPtr, &resource_id.to_bytes(), &old_version.to_be_bytes());
        }

        // Remove the rollback pointer itself.
        write_batch.delete(
            RollbackPtr,
            &concat_bytes!(&batch_index.to_be_bytes(), &resource_id.to_bytes()),
        );
    }
}
