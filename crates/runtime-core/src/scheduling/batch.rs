use crate::{
    BatchApi, RuntimeTx, Storage, Transaction, resources::resource_provider::ResourceProvider,
    utils::vec_ext::VecExt,
};

pub struct Batch<TX: Transaction> {
    transactions: Vec<RuntimeTx<TX>>,
    api: BatchApi<TX>,
}

impl<TX: Transaction> Batch<TX> {
    pub fn txs(&self) -> &[RuntimeTx<TX>] {
        &self.transactions
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
            transactions: txs.into_vec(|tx| RuntimeTx::new(api.clone(), resources, tx)),
            api,
        }
    }
}
