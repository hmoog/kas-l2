use std::sync::Arc;

use crate::{BatchAPI, ResourceProvider, ScheduledTransaction, Transaction};

pub struct Batch<T: Transaction> {
    scheduled_transactions: Vec<Arc<ScheduledTransaction<T>>>,
    api: Arc<BatchAPI<T>>,
}

impl<T: Transaction> Batch<T> {
    pub fn new(transactions: Vec<T>, resources: &mut ResourceProvider<T>) -> Self {
        let api = Arc::new(BatchAPI::new(transactions.len() as u64));
        Self {
            scheduled_transactions: transactions
                .into_iter()
                .map(|transaction| {
                    let resources = resources.provide(&transaction);
                    ScheduledTransaction::new(transaction, resources, api.clone())
                })
                .collect(),
            api,
        }
    }

    pub fn size(&self) -> usize {
        self.scheduled_transactions.len()
    }

    pub fn api(&self) -> Arc<BatchAPI<T>> {
        self.api.clone()
    }
}
