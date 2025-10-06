use crate::{BatchApi, ResourceProvider, RuntimeTx, StateDiff, Storage, Transaction, VecExt};

pub struct Batch<Tx: Transaction> {
    txs: Vec<RuntimeTx<Tx>>,
    state_diffs: Vec<StateDiff<Tx>>,
    api: BatchApi<Tx>,
}

impl<Tx: Transaction> Batch<Tx> {
    pub fn txs(&self) -> &[RuntimeTx<Tx>] {
        &self.txs
    }

    pub fn state_diffs(&self) -> &[StateDiff<Tx>] {
        &self.state_diffs
    }

    pub fn api(&self) -> &BatchApi<Tx> {
        &self.api
    }

    pub(crate) fn new<S: Storage<Tx::ResourceId>>(
        txs: Vec<Tx>,
        provider: &mut ResourceProvider<Tx, S>,
    ) -> Self {
        let api = BatchApi::new(txs.len());
        let mut state_diffs = Vec::new();
        Self {
            txs: txs.into_vec(|tx| RuntimeTx::new(provider, &mut state_diffs, api.downgrade(), tx)),
            state_diffs,
            api,
        }
    }
}
