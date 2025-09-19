use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crossbeam_deque::Injector;
use kas_l2_atomic::AtomicAsyncLatch;

use crate::{ScheduledTransaction, Transaction};

pub struct BatchAPI<T: Transaction> {
    pub scheduled_tasks: Injector<Arc<ScheduledTransaction<T>>>,
    pending_tasks: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> BatchAPI<T> {
    pub(crate) fn new(pending_tasks: u64) -> Self {
        Self {
            scheduled_tasks: Injector::new(),
            pending_tasks: AtomicU64::new(pending_tasks),
            is_done: AtomicAsyncLatch::new(),
        }
    }

    pub fn pending_tasks(&self) -> u64 {
        self.pending_tasks.load(Ordering::Acquire)
    }

    pub fn is_done(&self) -> bool {
        self.is_done.is_open()
    }

    pub fn wait(&self) -> impl Future<Output = ()> + '_ {
        self.is_done.wait()
    }

    pub(crate) fn decrease_pending_tasks(&self) {
        if self.pending_tasks.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.is_done.open();
        }
    }
}
