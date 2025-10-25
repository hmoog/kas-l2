use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_macros::smart_pointer;
use kas_l2_storage::Store;

use crate::{
    AccessHandle, BatchRef, ResourceAccess, RuntimeState, StateDiff, Transaction,
    TransactionProcessor, VecExt, scheduling::scheduler::Scheduler,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<S: Store<StateSpace = RuntimeState>, Tx: Transaction> {
    batch: BatchRef<S, Tx>,
    resources: Vec<ResourceAccess<S, Tx>>,
    pending_resources: AtomicU64,
    tx: Tx,
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> RuntimeTx<S, Tx> {
    pub fn accessed_resources(&self) -> &[ResourceAccess<S, Tx>] {
        &self.resources
    }

    pub(crate) fn new(
        scheduler: &mut Scheduler<S, Tx>,
        state_diffs: &mut Vec<StateDiff<S, Tx>>,
        batch: BatchRef<S, Tx>,
        tx: Tx,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<S, Tx>>| {
            let resources =
                scheduler.resources(&tx, RuntimeTxRef(this.clone()), &batch, state_diffs);
            RuntimeTxData {
                pending_resources: AtomicU64::new(resources.len() as u64),
                batch,
                tx,
                resources,
            }
        }))
    }

    pub(crate) fn execute<TxProc: TransactionProcessor<S, Tx>>(&self, processor: &TxProc) {
        if let Some(batch) = self.batch.upgrade() {
            let mut handles = self.resources.as_vec(AccessHandle::new);
            match processor(&self.tx, &mut handles) {
                Ok(()) => handles.into_iter().for_each(AccessHandle::commit_changes),
                Err(_) => handles.into_iter().for_each(AccessHandle::rollback_changes),
            }

            batch.decrease_pending_txs();
        }
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.pending_resources.fetch_sub(1, Ordering::Release) == 1 {
            if let Some(batch) = self.batch.upgrade() {
                batch.push_available_tx(&self)
            }
        }
    }

    pub(crate) fn batch(&self) -> &BatchRef<S, Tx> {
        &self.batch
    }
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> RuntimeTxRef<S, T> {
    pub(crate) fn belongs_to_batch(&self, batch: &BatchRef<S, T>) -> bool {
        self.upgrade().is_some_and(|tx| tx.batch() == batch)
    }
}
