use std::sync::Arc;

use kas_l2_runtime_core::{atomic::AtomicOptionArc, storage::KvStore, transactions::Transaction};

use crate::{BatchAPI, ResourceProvider, ScheduledTransaction};

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

    pub(crate) fn new<K: KvStore<T::ResourceID>>(
        prev: Option<Arc<Self>>,
        transactions: Vec<T>,
        resources: &mut ResourceProvider<T, K>,
    ) -> Self {
        let api = Arc::new(BatchAPI::new(transactions.len() as u64));
        let scheduled_transactions = transactions
            .into_iter()
            .map(|tx| ScheduledTransaction::new(resources.provide_resources(&tx), tx, api.clone()))
            .collect();

        Self {
            _prev: AtomicOptionArc::new(prev),
            scheduled_transactions,
            api,
        }
    }
}
