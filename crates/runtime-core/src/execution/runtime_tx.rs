use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_runtime_macros::smart_pointer;

use crate::{
    AccessHandle, BatchRef, ResourceAccess, ResourceProvider, StateDiff, Storage, Transaction,
    TransactionProcessor, VecExt,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<Tx: Transaction> {
    batch: BatchRef<Tx>,
    resources: Vec<ResourceAccess<Tx>>,
    pending_resources: AtomicU64,
    tx: Tx,
}

impl<Tx: Transaction> RuntimeTx<Tx> {
    pub fn accessed_resources(&self) -> &[ResourceAccess<Tx>] {
        &self.resources
    }

    pub(crate) fn new<TxStorage: Storage<Tx::ResourceId>>(
        provider: &mut ResourceProvider<Tx, TxStorage>,
        state_diffs: &mut Vec<StateDiff<Tx>>,
        batch: BatchRef<Tx>,
        tx: Tx,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<Tx>>| {
            let resources = provider.provide(&tx, RuntimeTxRef(this.clone()), &batch, state_diffs);
            RuntimeTxData {
                pending_resources: AtomicU64::new(resources.len() as u64),
                batch,
                tx,
                resources,
            }
        }))
    }

    pub(crate) fn execute<TxProc: TransactionProcessor<Tx>>(&self, processor: &TxProc) {
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
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            if let Some(batch) = self.batch.upgrade() {
                batch.push_available_tx(&self)
            }
        }
    }

    pub(crate) fn batch(&self) -> &BatchRef<Tx> {
        &self.batch
    }
}

impl<T: Transaction> RuntimeTxRef<T> {
    pub(crate) fn belongs_to_batch(&self, batch: &BatchRef<T>) -> bool {
        self.upgrade().is_some_and(|tx| tx.batch() == batch)
    }
}
