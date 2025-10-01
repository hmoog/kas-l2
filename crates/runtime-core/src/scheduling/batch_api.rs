use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_atomic::AtomicAsyncLatch;

use crate::{Transaction, scheduling::scheduled_transaction::ScheduledTransaction};

pub struct BatchAPI<T: Transaction> {
    available_transactions: Injector<Arc<ScheduledTransaction<T>>>,
    pending_transactions: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> BatchAPI<T> {
    pub fn available_transactions(&self) -> u64 {
        self.available_transactions.len() as u64
    }

    pub fn pending_transactions(&self) -> u64 {
        self.pending_transactions.load(Ordering::Acquire)
    }

    pub fn is_done(&self) -> bool {
        self.is_done.is_open()
    }

    pub fn wait_done(&self) -> impl Future<Output = ()> + '_ {
        self.is_done.wait()
    }

    pub(crate) fn new(transaction_count: usize) -> Arc<Self> {
        Arc::new(Self {
            available_transactions: Injector::new(),
            pending_transactions: AtomicU64::new(transaction_count as u64),
            is_done: AtomicAsyncLatch::new(),
        })
    }

    pub(crate) fn push_available(&self, transaction: &Arc<ScheduledTransaction<T>>) {
        self.available_transactions.push(transaction.clone());
    }

    pub(crate) fn steal_available_transactions(
        &self,
        worker: &Worker<Arc<ScheduledTransaction<T>>>,
    ) -> Steal<Arc<ScheduledTransaction<T>>> {
        self.available_transactions.steal_batch_and_pop(worker)
    }

    pub(crate) fn decrease_pending_transactions(&self) {
        if self.pending_transactions.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.is_done.open();
        }
    }
}
