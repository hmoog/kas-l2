use crate::{
    BatchApi, ScheduledTransaction, Storage, Transaction,
    resources::resource_provider::ResourceProvider,
};

pub struct Batch<T: Transaction> {
    transactions: Vec<ScheduledTransaction<T>>,
    api: BatchApi<T>,
}

impl<T: Transaction> Batch<T> {
    pub fn transactions(&self) -> &[ScheduledTransaction<T>] {
        &self.transactions
    }

    pub fn api(&self) -> &BatchApi<T> {
        &self.api
    }

    pub(crate) fn new<S: Storage<T::ResourceID>>(
        transactions: Vec<T>,
        resources: &mut ResourceProvider<T, S>,
    ) -> Self {
        let api = BatchApi::new(transactions.len());
        Self {
            transactions: map(transactions, |t| {
                ScheduledTransaction::new(api.clone(), resources, t)
            }),
            api,
        }
    }
}

fn map<Src, Dest, Mapping: FnMut(Src) -> Dest>(src: Vec<Src>, map: Mapping) -> Vec<Dest> {
    src.into_iter().map(map).collect()
}
