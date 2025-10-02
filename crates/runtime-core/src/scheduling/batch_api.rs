use std::{
    future::Future,
    sync::atomic::{AtomicU64, Ordering},
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_atomic::AtomicAsyncLatch;

use crate::{ScheduledTransaction, Transaction};

pub struct BatchApi<T: Transaction> {
    available_transactions: Injector<ScheduledTransaction<T>>,
    pending_transactions: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> BatchApi<T> {
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

    pub(crate) fn new(transaction_count: usize) -> Self {
        Self {
            available_transactions: Injector::new(),
            pending_transactions: AtomicU64::new(transaction_count as u64),
            is_done: AtomicAsyncLatch::new(),
        }
    }

    pub(crate) fn push_available(&self, transaction: &ScheduledTransaction<T>) {
        self.available_transactions.push(transaction.clone());
    }

    pub(crate) fn steal_available_transactions(
        &self,
        worker: &Worker<ScheduledTransaction<T>>,
    ) -> Option<ScheduledTransaction<T>> {
        loop {
            match self.available_transactions.steal_batch_and_pop(worker) {
                Steal::Success(task) => return Some(task),
                Steal::Retry => continue,
                Steal::Empty => return None,
            }
        }
    }

    pub(crate) fn decrease_pending_transactions(&self) {
        if self.pending_transactions.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.is_done.open();
        }
    }
}

pub(crate) mod shared {
    use std::{ops::Deref, sync::Arc};

    use crate::Transaction;

    pub struct BatchApi<T: Transaction>(Arc<super::BatchApi<T>>);

    impl<T: Transaction> BatchApi<T> {
        pub(crate) fn new(transaction_count: usize) -> Self {
            Self(Arc::new(super::BatchApi::new(transaction_count)))
        }
    }

    impl<T: Transaction> Deref for BatchApi<T> {
        type Target = super::BatchApi<T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Transaction> Clone for BatchApi<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
}
