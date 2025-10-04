use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_runtime_macros::smart_pointer;
use tap::Tap;

use crate::{
    AccessHandle, BatchApi, ResourceAccess, ResourceProvider, Storage, Transaction,
    TransactionProcessor, VecExt,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<Tx: Transaction> {
    batch_api: BatchApi<Tx>,
    resources: Vec<ResourceAccess<Tx>>,
    pending_resources: AtomicU64,
    tx: Tx,
}

impl<Tx: Transaction> RuntimeTx<Tx> {
    pub fn accessed_resources(&self) -> &[ResourceAccess<Tx>] {
        &self.resources
    }

    pub(crate) fn new<TxStorage: Storage<Tx::ResourceId>>(
        batch_api: BatchApi<Tx>,
        resources: &mut ResourceProvider<Tx, TxStorage>,
        tx: Tx,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<Tx>>| {
            let resources = resources.provide(&tx, RuntimeTxRef(this.clone()));
            RuntimeTxData {
                pending_resources: AtomicU64::new(resources.len() as u64),
                batch_api,
                tx,
                resources,
            }
        }))
        .tap(|this| {
            for resource in this.accessed_resources() {
                resource.init(resources);
            }
        })
    }

    pub(crate) fn batch_api(&self) -> &BatchApi<Tx> {
        &self.batch_api
    }

    pub(crate) fn execute<TxProc: TransactionProcessor<Tx>>(&self, processor: &TxProc) {
        let mut handles = self.resources.as_vec(AccessHandle::new);
        match processor(&self.tx, &mut handles) {
            Ok(()) => handles.into_iter().for_each(AccessHandle::commit_changes),
            Err(_) => handles.into_iter().for_each(AccessHandle::rollback_changes),
        }

        self.batch_api.decrease_pending_txs();
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.batch_api.push_available_tx(&self)
        }
    }
}
