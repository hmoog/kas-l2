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
    scheduled_transactions: Injector<Arc<ScheduledTransaction<T>>>,
    pending_transactions: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> BatchAPI<T> {
    pub(crate) fn steal_scheduled_transactions(
        &self,
        dest: &Worker<Arc<ScheduledTransaction<T>>>,
    ) -> Steal<Arc<ScheduledTransaction<T>>> {
        self.scheduled_transactions.steal_batch_and_pop(dest)
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

    pub(crate) fn new(pending_tasks: u64) -> Self {
        Self {
            scheduled_transactions: Injector::new(),
            pending_transactions: AtomicU64::new(pending_tasks),
            is_done: AtomicAsyncLatch::new(),
        }
    }

    pub(crate) fn schedule_transaction(&self, transaction: Arc<ScheduledTransaction<T>>) {
        self.scheduled_transactions.push(transaction);
    }

    pub(crate) fn decrease_pending_transactions(&self) {
        if self.pending_transactions.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.is_done.open();
        }
    }
}
