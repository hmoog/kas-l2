use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_atomic::AtomicAsyncLatch;
use kas_l2_macros::smart_pointer;
use kas_l2_storage::{Storage, Store, WriteStore};
use tap::Tap;

use crate::{
    ResourceProvider, RuntimeTx, StateDiff, Transaction, VecExt,
    storage::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

#[smart_pointer]
pub struct Batch<S: Store<StateSpace = RuntimeState>, Tx: Transaction> {
    index: u64,
    storage: Storage<S, Read<S, Tx>, Write<S, Tx>>,
    txs: Vec<RuntimeTx<S, Tx>>,
    state_diffs: Vec<StateDiff<S, Tx>>,
    available_txs: Injector<RuntimeTx<S, Tx>>,
    pending_txs: AtomicU64,
    pending_writes: AtomicI64,
    was_processed: AtomicAsyncLatch,
    was_persisted: AtomicAsyncLatch,
    was_committed: AtomicAsyncLatch,
}

impl<S: Store<StateSpace = RuntimeState>, Tx: Transaction> Batch<S, Tx> {
    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn txs(&self) -> &[RuntimeTx<S, Tx>] {
        &self.txs
    }

    pub fn state_diffs(&self) -> &[StateDiff<S, Tx>] {
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

    pub(crate) fn new(
        index: u64,
        storage: &Storage<S, Read<S, Tx>, Write<S, Tx>>,
        txs: Vec<Tx>,
        provider: &mut ResourceProvider<S, Tx>,
    ) -> Self {
        Self(Arc::new_cyclic(|this| {
            let mut state_diffs = Vec::new();
            BatchData {
                index,
                storage: storage.clone(),
                pending_txs: AtomicU64::new(txs.len() as u64),
                pending_writes: AtomicI64::new(0),
                txs: txs.into_vec(|tx| {
                    RuntimeTx::new(provider, &mut state_diffs, BatchRef(this.clone()), tx)
                }),
                state_diffs,
                available_txs: Injector::new(),
                was_processed: Default::default(),
                was_persisted: Default::default(),
                was_committed: Default::default(),
            }
        }))
        .tap(|this| {
            for tx in this.txs() {
                for resource in tx.accessed_resources() {
                    resource.init(storage);
                }
            }
        })
    }

    pub(crate) fn push_available_tx(&self, tx: &RuntimeTx<S, Tx>) {
        self.available_txs.push(tx.clone());
    }

    pub(crate) fn steal_available_txs(
        &self,
        worker: &Worker<RuntimeTx<S, Tx>>,
    ) -> Option<RuntimeTx<S, Tx>> {
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

    pub(crate) fn submit_write(&self, write: Write<S, Tx>) {
        self.pending_writes.fetch_add(1, Ordering::AcqRel);
        self.storage.submit_write(write);
    }

    pub(crate) fn decrease_pending_writes(&self) {
        if self.pending_writes.fetch_sub(1, Ordering::AcqRel) == 1 && self.num_pending() == 0 {
            self.was_persisted.open();
            self.storage.submit_write(Write::PointerFlip(self.clone()));
        }
    }

    pub(crate) fn write_pointer_flip<W>(&self, store: &mut W)
    where
        W: WriteStore<StateSpace = RuntimeState>,
    {
        for state_diff in self.state_diffs() {
            state_diff.written_state().write_latest_ptr(store);
        }
    }

    pub(crate) fn mark_committed(self) {
        // TODO: EVICT STUFF
        self.was_committed.open();
    }
}
