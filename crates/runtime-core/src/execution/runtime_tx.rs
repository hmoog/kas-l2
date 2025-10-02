use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_runtime_macros::smart_pointer;
use tap::Tap;

use crate::{
    BatchApi, RuntimeTxRef, Storage, Transaction, TransactionProcessor,
    resources::{
        accessed_resource::AccessedResource, resource_handle::ResourceHandle,
        resource_provider::ResourceProvider,
    },
    utils::vec_ext::VecExt,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<Tx: Transaction> {
    batch: BatchApi<Tx>,
    accessed_resources: Vec<AccessedResource<Tx>>,
    pending_resources: AtomicU64,
    tx: Tx,
}

impl<Tx: Transaction> RuntimeTx<Tx> {
    pub fn accessed_resources(&self) -> &[AccessedResource<Tx>] {
        &self.accessed_resources
    }

    pub(crate) fn new<TxStorage: Storage<Tx::ResourceID>>(
        batch: BatchApi<Tx>,
        resources: &mut ResourceProvider<Tx, TxStorage>,
        tx: Tx,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<Tx>>| {
            let accessed_resources = resources.provide(&tx, RuntimeTxRef(this.clone()));
            let pending_resources = AtomicU64::new(accessed_resources.len() as u64);
            RuntimeTxData {
                pending_resources,
                batch,
                tx,
                accessed_resources,
            }
        }))
        .tap(|this| {
            for resource in this.accessed_resources() {
                resource.init(|r| resources.load_from_storage(r));
            }
        })
    }

    pub(crate) fn execute<TxProc: TransactionProcessor<Tx>>(&self, processor: &TxProc) {
        let mut handles = self.accessed_resources.as_vec(ResourceHandle::new);
        processor(&self.tx, &mut handles);
        self.batch.decrease_pending_txs();
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.batch.push_available_tx(&self)
        }
    }
}
