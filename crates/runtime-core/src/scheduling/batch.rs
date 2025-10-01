use std::sync::Arc;

use crate::{
    BatchAPI, Storage, Transaction, resources::resource_provider::ResourceProvider,
    scheduling::scheduled_transaction::ScheduledTransaction,
};

pub struct Batch<T: Transaction> {
    transactions: Vec<Arc<ScheduledTransaction<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> Batch<T> {
    pub fn transactions(&self) -> &[Arc<ScheduledTransaction<T>>] {
        &self.transactions
    }

    pub fn api(&self) -> &Arc<BatchAPI<T>> {
        &self.api
    }

    pub(crate) fn new<S: Storage<T::ResourceID>>(
        transactions: Vec<T>,
        resources: &mut ResourceProvider<T, S>,
    ) -> Self {
        let api = BatchAPI::new(transactions.len());
        let transactions = vec_map(transactions, |t| {
            ScheduledTransaction::new(api.clone(), resources, t)
        });
        Self { transactions, api }
    }
}

fn vec_map<Src, Dest, Mapping: FnMut(Src) -> Dest>(src: Vec<Src>, map: Mapping) -> Vec<Dest> {
    src.into_iter().map(map).collect()
}
