use crate::{
    BatchApi, RuntimeTx, Storage, Transaction, resources::resource_provider::ResourceProvider,
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
            transactions: map(txs, |t| RuntimeTx::new(api.clone(), resources, t)),
            api,
        }
    }
}

fn map<Src, Dest, Mapping: FnMut(Src) -> Dest>(src: Vec<Src>, map: Mapping) -> Vec<Dest> {
    src.into_iter().map(map).collect()
}
