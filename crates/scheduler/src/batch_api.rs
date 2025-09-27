use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_runtime_core::{atomic::AtomicAsyncLatch, transactions::Transaction};

use crate::ScheduledTransaction;

pub struct BatchAPI<T: Transaction> {
    scheduled_tasks: Injector<Arc<ScheduledTransaction<T>>>,
    pending_tasks: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> BatchAPI<T> {
    pub fn steal_scheduled_transactions(
        &self,
        dest: &Worker<Arc<ScheduledTransaction<T>>>,
    ) -> Steal<Arc<ScheduledTransaction<T>>> {
        self.scheduled_tasks.steal_batch_and_pop(dest)
    }

    pub fn pending_transactions(&self) -> u64 {
        self.pending_tasks.load(Ordering::Acquire)
    }

    pub fn is_done(&self) -> bool {
        self.is_done.is_open()
    }

    pub fn wait_done(&self) -> impl Future<Output = ()> + '_ {
        self.is_done.wait()
    }

    pub(crate) fn new(pending_tasks: u64) -> Self {
        Self {
            scheduled_tasks: Injector::new(),
            pending_tasks: AtomicU64::new(pending_tasks),
            is_done: AtomicAsyncLatch::new(),
        }
    }

    pub(crate) fn schedule_transaction(&self, tx: Arc<ScheduledTransaction<T>>) {
        self.scheduled_tasks.push(tx);
    }

    pub(crate) fn transaction_done(&self) {
        if self.pending_tasks.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.is_done.open();
        }
    }
}
