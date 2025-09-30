use std::sync::{Arc, Weak};

use crate::{
    BatchAPI, Storage, Transaction, resources::resource_provider::ResourceProvider,
    scheduling::scheduled_transaction::ScheduledTransaction,
};

pub struct Batch<T: Transaction> {
    scheduled_transactions: Vec<Arc<ScheduledTransaction<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> Batch<T> {
    pub fn scheduled_transactions(&self) -> &[Arc<ScheduledTransaction<T>>] {
        &self.scheduled_transactions
    }

    pub fn api(&self) -> Arc<BatchAPI<T>> {
        self.api.clone()
    }

    pub(crate) fn new<K: Storage<T::ResourceID>>(
        transactions: Vec<T>,
        resources: &mut ResourceProvider<T, K>,
    ) -> Self {
        let api = Arc::new(BatchAPI::new(transactions.len() as u64));
        let _scheduled_transactions = transactions
            .into_iter()
            .map(|tx| {
                let scheduled_transaction =
                    Arc::new_cyclic(|scheduled_transaction: &Weak<ScheduledTransaction<T>>| {
                        ScheduledTransaction::new(
                            resources.provide_resources(&tx, scheduled_transaction, api.clone()),
                            tx,
                            api.clone(),
                        )
                    });

                for resource in scheduled_transaction.resources().into_iter() {
                    match resource.prev() {
                        Some(prev) => prev.set_next(resource),
                        None => resources.load_from_storage(resource),
                    }
                }

                scheduled_transaction
            })
            .collect();

        Self {
            scheduled_transactions: _scheduled_transactions,
            api,
        }
    }
}
