use std::sync::Arc;

use crate::{BatchAPI, ResourceProvider, ScheduledTransaction, Transaction};

pub struct Batch<T: Transaction> {
    scheduled_transactions: Vec<Arc<ScheduledTransaction<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> Batch<T> {
    pub fn new(transactions: Vec<T>, resources: &mut ResourceProvider<T>) -> Self {
        let api = Arc::new(BatchAPI::new(transactions.len() as u64));
        let scheduled_transactions = transactions
            .into_iter()
            .map(|transaction| {
                let resources = resources.provide(&transaction);
                ScheduledTransaction::new(transaction, resources, api.clone())
            })
            .collect();

        Self {
            scheduled_transactions,
            api,
        }
    }

    pub fn scheduled_transactions(&self) -> &Vec<Arc<ScheduledTransaction<T>>> {
        &self.scheduled_transactions
    }

    pub fn api(&self) -> &Arc<BatchAPI<T>> {
        &self.api
    }
}
