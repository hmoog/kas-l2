use crate::{BatchApi, ResourceProvider, RuntimeTx, Storage, Transaction, VecExt};

pub struct Batch<TX: Transaction> {
    txs: Vec<RuntimeTx<TX>>,
    api: BatchApi<TX>,
}

impl<TX: Transaction> Batch<TX> {
    pub fn txs(&self) -> &[RuntimeTx<TX>] {
        &self.txs
    }

    pub fn api(&self) -> &BatchApi<TX> {
        &self.api
    }

    pub(crate) fn new<S: Storage<TX::ResourceID>>(
        txs: Vec<TX>,
        resources: &mut ResourceProvider<TX, S>,
    ) -> Self {
        let api = BatchApi::new(txs.len());
        Self {
            txs: txs.into_vec(|tx| RuntimeTx::new(api.clone(), resources, tx)),
            api,
        }
    }
}
