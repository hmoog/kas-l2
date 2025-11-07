use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_core_atomics::AtomicAsyncLatch;
use kas_l2_core_macros::smart_pointer;
use kas_l2_storage_manager::{StorageManager, Store, WriteStore};

use crate::{
    Read, RuntimeTx, Scheduler, StateDiff, VecExt, Write, storage::runtime_state::RuntimeState,
    vm::VM,
};

#[smart_pointer]
pub struct Batch<S: Store<StateSpace = RuntimeState>, V: VM> {
    index: u64,
    storage: StorageManager<S, Read<S, V>, Write<S, V>>,
    txs: Vec<RuntimeTx<S, V>>,
    state_diffs: Vec<StateDiff<S, V>>,
    available_txs: Injector<RuntimeTx<S, V>>,
    pending_txs: AtomicU64,
    pending_writes: AtomicI64,
    was_processed: AtomicAsyncLatch,
    was_persisted: AtomicAsyncLatch,
    was_committed: AtomicAsyncLatch,
}

impl<S: Store<StateSpace = RuntimeState>, V: VM> Batch<S, V> {
    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn txs(&self) -> &[RuntimeTx<S, V>] {
        &self.txs
    }

    pub fn state_diffs(&self) -> &[StateDiff<S, V>] {
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

    pub fn was_persisted(&self) -> bool {
        self.was_persisted.is_open()
    }

    pub fn wait_persisted(&self) -> impl Future<Output = ()> + '_ {
        self.was_persisted.wait()
    }

    pub fn was_committed(&self) -> bool {
        self.was_committed.is_open()
    }

    pub fn wait_committed(&self) -> impl Future<Output = ()> + '_ {
        self.was_committed.wait()
    }

    pub(crate) fn new(scheduler: &mut Scheduler<S, V>, txs: Vec<V::Transaction>) -> Self {
        Self(Arc::new_cyclic(|this| {
            let mut state_diffs = Vec::new();
            BatchData {
                index: scheduler.batch_index(),
                storage: scheduler.storage().clone(),
                pending_txs: AtomicU64::new(txs.len() as u64),
                pending_writes: AtomicI64::new(0),
                txs: txs.into_vec(|tx| {
                    RuntimeTx::new(scheduler, &mut state_diffs, BatchRef(this.clone()), tx)
                }),
                state_diffs,
                available_txs: Injector::new(),
                was_processed: Default::default(),
                was_persisted: Default::default(),
                was_committed: Default::default(),
            }
        }))
    }

    pub(crate) fn connect(&self) {
        for tx in self.txs() {
            for resource in tx.accessed_resources() {
                resource.connect(&self.storage);
            }
        }
    }

    pub(crate) fn push_available_tx(&self, tx: &RuntimeTx<S, V>) {
        self.available_txs.push(tx.clone());
    }

    pub(crate) fn steal_available_txs(
        &self,
        worker: &Worker<RuntimeTx<S, V>>,
    ) -> Option<RuntimeTx<S, V>> {
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

    pub(crate) fn submit_write(&self, write: Write<S, V>) {
        self.pending_writes.fetch_add(1, Ordering::AcqRel);
        self.storage.submit_write(write);
    }

    pub(crate) fn decrease_pending_writes(&self) {
        // TODO: CHECK IF THERE CAN BE A RACE BETWEEN PENDING_TXS AND PENDING_WRITES
        if self.pending_writes.fetch_sub(1, Ordering::AcqRel) == 1 && self.num_pending() == 0 {
            self.was_persisted.open();
        }
    }

    pub(crate) fn schedule_commit(&self) {
        self.storage.submit_write(Write::CommitBatch(self.clone()));
    }

    pub(crate) fn commit<W>(&self, store: &mut W)
    where
        W: WriteStore<StateSpace = RuntimeState>,
    {
        for state_diff in self.state_diffs() {
            state_diff.written_state().write_latest_ptr(store);
        }
    }

    pub(crate) fn commit_done(self) {
        // TODO: EVICT STUFF?
        self.was_committed.open();
    }
}
