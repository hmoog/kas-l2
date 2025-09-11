use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crossbeam_deque::Injector;
use kas_l2_atomic::AtomicAsyncLatch;

use crate::{ScheduledTask, Task};

pub struct PendingTasks<T: Task> {
    pub ready: Injector<Arc<ScheduledTask<T>>>,
    pub(crate) count: AtomicU64,
    pub(crate) done: AtomicAsyncLatch,
}

impl<T: Task> PendingTasks<T> {
    pub(crate) fn new(pending_tasks: u64) -> Self {
        Self {
            ready: Injector::new(),
            count: AtomicU64::new(pending_tasks),
            done: AtomicAsyncLatch::new(),
        }
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Acquire)
    }

    pub fn is_done(&self) -> bool {
        self.done.is_open()
    }

    pub fn wait(&self) -> impl Future<Output = ()> + '_ {
        self.done.wait()
    }
}
