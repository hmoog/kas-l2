use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use tap::Tap;

use crate::{
    BatchApi, Storage, Transaction, TransactionProcessor,
    resources::{
        accessed_resource::AccessedResource, resource_handle::ResourceHandle,
        resource_provider::ResourceProvider,
    },
};

pub struct RuntimeTx<Tx: Transaction>(Arc<RuntimeTxData<Tx>>);

struct RuntimeTxData<Tx: Transaction> {
    batch: BatchApi<Tx>,
    resources: Vec<Arc<AccessedResource<Tx>>>,
    pending_resources: AtomicU64,
    tx: Tx,
}

impl<Tx: Transaction> RuntimeTx<Tx> {
    pub fn downgrade(&self) -> RuntimeTxRef<Tx> {
        RuntimeTxRef(Arc::downgrade(&self.0))
    }

    pub(crate) fn new<TxStorage: Storage<Tx::ResourceID>>(
        batch: BatchApi<Tx>,
        resources: &mut ResourceProvider<Tx, TxStorage>,
        tx: Tx,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<Tx>>| {
            let resources = resources.provide(&tx, RuntimeTxRef(this.clone()));
            RuntimeTxData {
                pending_resources: AtomicU64::new(resources.len() as u64),
                batch,
                tx,
                resources,
            }
        }))
        .tap(|this| {
            for resource in this.resources() {
                resource.init(|r| resources.load_from_storage(r));
            }
        })
    }

    pub fn resources(&self) -> &[Arc<AccessedResource<Tx>>] {
        &self.0.resources
    }

    pub(crate) fn process<F: TransactionProcessor<Tx>>(&self, processor: &F) {
        processor(&self.0.tx, &mut self.handles());
        self.0.batch.decrease_pending_txs();
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.0.pending_resources.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.batch.push_available_tx(&self)
        }
    }

    fn handles(&self) -> Vec<ResourceHandle<'_, Tx>> {
        self.0.resources.iter().map(ResourceHandle::new).collect()
    }
}

impl<T: Transaction> Clone for RuntimeTx<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub struct RuntimeTxRef<T: Transaction>(Weak<RuntimeTxData<T>>);

impl<T: Transaction> RuntimeTxRef<T> {
    pub fn upgrade(&self) -> Option<RuntimeTx<T>> {
        self.0.upgrade().map(RuntimeTx)
    }
}

impl<T: Transaction> PartialEq for RuntimeTxRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Transaction> Clone for RuntimeTxRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
