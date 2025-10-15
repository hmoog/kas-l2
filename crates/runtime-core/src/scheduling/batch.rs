use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_io::{IoManager, Storage};
use kas_l2_runtime_macros::smart_pointer;
use tap::Tap;

use crate::{
    ResourceProvider, RuntimeTx, StateDiff, Transaction, VecExt,
    io::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

#[smart_pointer]
pub struct Batch<Tx: Transaction> {
    txs: Vec<RuntimeTx<Tx>>,
    state_diffs: Vec<StateDiff<Tx>>,
    available_txs: Injector<RuntimeTx<Tx>>,
    pending_txs: AtomicU64,
    pending_writes: AtomicI64,
    was_processed: AtomicAsyncLatch,
    // was_persisted: AtomicAsyncLatch,
    // was_committed: AtomicAsyncLatch,
}

impl<Tx: Transaction> Batch<Tx> {
    pub fn txs(&self) -> &[RuntimeTx<Tx>] {
        &self.txs
    }

    pub fn state_diffs(&self) -> &[StateDiff<Tx>] {
        &self.state_diffs
    }

    pub fn num_available(&self) -> u64 {
        self.available_txs.len() as u64
    }

    pub fn num_pending(&self) -> u64 {
        self.pending_txs.load(Ordering::Acquire)
    }

    pub fn is_depleted(&self) -> bool {
        self.num_pending() == 0 && self.available_txs.is_empty()
    }

    pub fn was_processed(&self) -> bool {
        self.was_processed.is_open()
    }

    pub fn wait_processed(&self) -> impl Future<Output = ()> + '_ {
        self.was_processed.wait()
    }

    pub(crate) fn new<S: Storage<StateSpace = RuntimeState>>(
        io: &IoManager<S, Read<Tx>, Write<Tx>>,
        txs: Vec<Tx>,
        provider: &mut ResourceProvider<Tx>,
    ) -> Self {
        Self(Arc::new_cyclic(|this| {
            let mut state_diffs = Vec::new();
            BatchData {
                pending_txs: AtomicU64::new(txs.len() as u64),
                pending_writes: AtomicI64::new(0),
                txs: txs.into_vec(|tx| {
                    RuntimeTx::new(provider, &mut state_diffs, BatchRef(this.clone()), tx)
                }),
                state_diffs,
                available_txs: Injector::new(),
                was_processed: Default::default(),
                // was_persisted: Default::default(),
                // was_committed: Default::default(),
            }
        }))
        .tap(|this| {
            for tx in this.txs() {
                for resource in tx.accessed_resources() {
                    resource.init(io);
                }
            }

            // TOD: REMOVE REMOVE REMOVE
            let _cmd = Write::Batch(this.clone());
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
            self.was_processed.open();
        }
    }

    pub(crate) fn increase_pending_writes(&self) {
        self.pending_writes.fetch_add(1, Ordering::AcqRel);
    }

    // pub(crate) fn decrease_pending_writes(&self) {
    //     if self.pending_writes.fetch_sub(1, Ordering::AcqRel) == 1 && self.num_pending() == 0 {
    //         self.was_persisted.open();
    //     }
    // }
}
