use std::sync::Arc;

use kas_l2_atomic::AtomicOptionArc;

use crate::{BatchAPI, ResourcesManager, ScheduledTransaction, Transaction};

pub struct Batch<T: Transaction> {
    scheduled_transactions: Vec<Arc<ScheduledTransaction<T>>>,
    api: Arc<BatchAPI<T>>,
    _prev: AtomicOptionArc<Self>,
}

impl<T: Transaction> Batch<T> {
    pub fn scheduled_transactions(&self) -> &[Arc<ScheduledTransaction<T>>] {
        &self.scheduled_transactions
    }

    pub fn api(&self) -> Arc<BatchAPI<T>> {
        self.api.clone()
    }

    pub(crate) fn new(
        prev: Option<Arc<Self>>,
        transactions: Vec<T>,
        resources: &mut ResourcesManager<T>,
    ) -> Self {
        let api = Arc::new(BatchAPI::new(transactions.len() as u64));
        let scheduled_transactions = transactions
            .into_iter()
            .map(|tx| ScheduledTransaction::new(resources.provide(&tx), tx, api.clone()))
            .collect();

        Self {
            _prev: AtomicOptionArc::new(prev),
            scheduled_transactions,
            api,
        }
    }
}
