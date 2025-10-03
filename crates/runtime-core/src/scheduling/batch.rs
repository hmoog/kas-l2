use crate::{BatchApi, ResourceProvider, RuntimeTx, Storage, Transaction, VecExt};

pub struct Batch<Tx: Transaction> {
    txs: Vec<RuntimeTx<Tx>>,
    api: BatchApi<Tx>,
}

impl<Tx: Transaction> Batch<Tx> {
    pub fn txs(&self) -> &[RuntimeTx<Tx>] {
        &self.txs
    }

    pub fn api(&self) -> &BatchApi<Tx> {
        &self.api
    }

    pub(crate) fn new<S: Storage<Tx::ResourceId>>(
        txs: Vec<Tx>,
        resources: &mut ResourceProvider<Tx, S>,
    ) -> Self {
        let api = BatchApi::new(txs.len());
        Self {
            txs: txs.into_vec(|tx| RuntimeTx::new(api.clone(), resources, tx)),
            api,
        }
    }
}
