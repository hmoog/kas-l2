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
use tap::Tap;

use crate::{ResourceProvider, RuntimeTx, StateDiff, Storage, Transaction, VecExt};

#[smart_pointer]
pub struct Batch<Tx: Transaction> {
    txs: Vec<RuntimeTx<Tx>>,
    state_diffs: Vec<StateDiff<Tx>>,
    available_txs: Injector<RuntimeTx<Tx>>,
    pending_txs: AtomicU64,
    is_done: AtomicAsyncLatch,
}

impl<Tx: Transaction> Batch<Tx> {
    pub fn txs(&self) -> &[RuntimeTx<Tx>] {
        &self.txs
    }

    pub fn state_diffs(&self) -> &[StateDiff<Tx>] {
        &self.state_diffs
    }

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

    pub(crate) fn new<S: Storage<Tx::ResourceId>>(
        txs: Vec<Tx>,
        provider: &mut ResourceProvider<Tx, S>,
    ) -> Self {
        Self(Arc::new_cyclic(|this| {
            let mut state_diffs = Vec::new();
            BatchData {
                pending_txs: AtomicU64::new(txs.len() as u64),
                txs: txs.into_vec(|tx| {
                    RuntimeTx::new(provider, &mut state_diffs, BatchRef(this.clone()), tx)
                }),
                state_diffs,
                available_txs: Injector::new(),
                is_done: Default::default(),
            }
        }))
        .tap(|this| {
            for tx in this.txs() {
                for resource in tx.accessed_resources() {
                    resource.init(provider);
                }
            }
        })
    }

    pub(crate) fn push_available_tx(&self, tx: &RuntimeTx<Tx>) {
        self.available_txs.push(tx.clone());
    }

    pub(crate) fn steal_available_txs(
        &self,
        worker: &Worker<RuntimeTx<Tx>>,
    ) -> Option<RuntimeTx<Tx>> {
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
