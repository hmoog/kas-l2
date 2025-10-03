use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_runtime_macros::smart_pointer;

use crate::{RuntimeTx, Transaction};

#[smart_pointer]
pub struct BatchApi<T: Transaction> {
    available_txs: Injector<RuntimeTx<T>>,
    pending_txs: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<T: Transaction> BatchApi<T> {
    pub fn available_txs(&self) -> u64 {
        self.available_txs.len() as u64
    }

    pub fn pending_txs(&self) -> u64 {
        self.pending_txs.load(Ordering::Acquire)
    }

    pub fn is_depleted(&self) -> bool {
        self.pending_txs.load(Ordering::Acquire) == 0 && self.available_txs.is_empty()
    }

    pub fn is_done(&self) -> bool {
        self.is_done.is_open()
    }

    pub fn wait_done(&self) -> impl Future<Output = ()> + '_ {
        self.is_done.wait()
    }

    pub(crate) fn new(tx_count: usize) -> Self {
        Self(Arc::new(BatchApiData {
            available_txs: Injector::new(),
            pending_txs: AtomicU64::new(tx_count as u64),
            is_done: AtomicAsyncLatch::new(),
        }))
    }

    pub(crate) fn push_available_tx(&self, tx: &RuntimeTx<T>) {
        self.available_txs.push(tx.clone());
    }

    pub(crate) fn steal_available_txs(
        &self,
        worker: &Worker<RuntimeTx<T>>,
    ) -> Option<RuntimeTx<T>> {
        loop {
            match self.available_txs.steal_batch_and_pop(worker) {
                Steal::Success(task) => return Some(task),
                Steal::Retry => continue,
                Steal::Empty => return None,
            }
        }
    }

    pub(crate) fn decrease_pending_txs(&self) {
        if self.pending_txs.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.is_done.open();
        }
    }
}
