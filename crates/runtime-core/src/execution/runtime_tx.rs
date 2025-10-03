use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_runtime_macros::smart_pointer;
use tap::Tap;

use crate::{
    AccessedResource, BatchApi, ResourceHandle, ResourceProvider, RuntimeTxRef, Storage,
    Transaction, TransactionProcessor, VecExt,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<Tx: Transaction> {
    batch_api: BatchApi<Tx>,
    accessed_resources: Vec<AccessedResource<Tx>>,
    pending_resources: AtomicU64,
    tx: Tx,
}

impl<Tx: Transaction> RuntimeTx<Tx> {
    pub fn accessed_resources(&self) -> &[AccessedResource<Tx>] {
        &self.accessed_resources
    }

    pub(crate) fn new<TxStorage: Storage<Tx::ResourceId>>(
        batch_api: BatchApi<Tx>,
        resources: &mut ResourceProvider<Tx, TxStorage>,
        tx: Tx,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<Tx>>| {
            let accessed_resources = resources.provide(&tx, RuntimeTxRef(this.clone()));
            let pending_resources = AtomicU64::new(accessed_resources.len() as u64);
            RuntimeTxData {
                pending_resources,
                batch_api,
                tx,
                accessed_resources,
            }
        }))
        .tap(|this| {
            for resource in this.accessed_resources() {
                resource.init(resources);
            }
        })
    }

    pub(crate) fn execute<TxProc: TransactionProcessor<Tx>>(&self, processor: &TxProc) {
        let mut handles = self.accessed_resources.as_vec(ResourceHandle::new);
        processor(&self.tx, &mut handles);
        self.batch_api.decrease_pending_txs();
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.batch_api.push_available_tx(&self)
        }
    }
}
