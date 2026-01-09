use std::sync::{
    Arc,
    atomic::{AtomicI64, AtomicU64, Ordering},
};

use crossbeam_deque::{Injector, Steal, Worker};
use kas_l2_core_atomics::AtomicAsyncLatch;
use kas_l2_core_macros::smart_pointer;
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_manager::StorageManager;
use kas_l2_storage_types::{Store, WriteStore};

use crate::{
    Chain, Read, RuntimeManager, RuntimeTx, StateDiff, Write, cpu_task::ManagerTask,
    vm_interface::VmInterface,
};

#[smart_pointer]
pub struct RuntimeBatch<S: Store<StateSpace = StateSpace>, V: VmInterface> {
    chain: Chain,
    index: u64,
    storage: StorageManager<S, Read<S, V>, Write<S, V>>,
    txs: Vec<RuntimeTx<S, V>>,
    state_diffs: Vec<StateDiff<S, V>>,
    available_txs: Injector<ManagerTask<S, V>>,
    pending_txs: AtomicU64,
    pending_writes: AtomicI64,
    was_processed: AtomicAsyncLatch,
    was_persisted: AtomicAsyncLatch,
    was_committed: AtomicAsyncLatch,
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> RuntimeBatch<S, V> {
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

    pub fn was_canceled(&self) -> bool {
        self.index > self.chain.rollback_threshold()
    }

    pub fn was_processed(&self) -> bool {
        self.was_processed.is_open()
    }

    pub async fn wait_processed(&self) {
        if self.was_canceled() {
            return;
        }

        self.was_processed.wait().await
    }

    pub fn wait_processed_blocking(&self) -> &Self {
        self.was_processed.wait_blocking();
        self
    }

    pub fn was_persisted(&self) -> bool {
        self.was_persisted.is_open()
    }

    pub async fn wait_persisted(&self) {
        if self.was_canceled() {
            return;
        }

        self.was_persisted.wait().await
    }

    pub fn wait_persisted_blocking(&self) -> &Self {
        self.was_persisted.wait_blocking();
        self
    }

    pub fn was_committed(&self) -> bool {
        self.was_committed.is_open()
    }

    pub async fn wait_committed(&self) {
        if self.was_canceled() {
            return;
        }

        self.was_committed.wait().await
    }

    pub fn wait_committed_blocking(&self) -> &Self {
        self.was_committed.wait_blocking();
        self
    }

    pub(crate) fn new(
        vm: V,
        scheduler: &mut RuntimeManager<S, V>,
        txs: Vec<V::Transaction>,
    ) -> Self {
        Self(Arc::new_cyclic(|this| {
            let mut state_diffs = Vec::new();
            let chain = scheduler.longest_chain().clone();
            RuntimeBatchData {
                index: chain.next_batch_index(),
                storage: scheduler.storage_manager().clone(),
                pending_txs: AtomicU64::new(txs.len() as u64),
                pending_writes: AtomicI64::new(0),
                txs: txs
                    .into_iter()
                    .map(|tx| {
                        RuntimeTx::new(
                            &vm,
                            scheduler,
                            &mut state_diffs,
                            RuntimeBatchRef(this.clone()),
                            tx,
                        )
                    })
                    .collect(),
                state_diffs,
                chain,
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
        self.available_txs.push(ManagerTask::ExecuteTransaction(tx.clone()));
    }

    pub(crate) fn decrease_pending_txs(&self) {
        if self.pending_txs.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.was_processed.open();
        }
    }

    pub(crate) fn submit_write(&self, write: Write<S, V>) {
        if !self.was_canceled() {
            self.pending_writes.fetch_add(1, Ordering::AcqRel);
            self.storage.submit_write(write);
        }
    }

    pub(crate) fn decrease_pending_writes(&self) {
        // TODO: CHECK IF THERE CAN BE A RACE BETWEEN PENDING_TXS AND PENDING_WRITES
        if self.pending_writes.fetch_sub(1, Ordering::AcqRel) == 1 && self.num_pending() == 0 {
            self.was_persisted.open();
        }
    }

    pub fn schedule_commit(&self) {
        if !self.was_canceled() {
            self.storage.submit_write(Write::CommitBatch(self.clone()));
        }
    }

    pub(crate) fn commit<W>(&self, store: &mut W)
    where
        W: WriteStore<StateSpace = StateSpace>,
    {
        if !self.was_canceled() {
            for state_diff in self.state_diffs() {
                state_diff.written_state().write_latest_ptr(store);
            }
        }
    }

    pub(crate) fn commit_done(self) {
        // TODO: EVICT STUFF FROM STORAGE MANAGER
        self.was_committed.open();
    }
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface>
    kas_l2_runtime_execution_workers::Batch<ManagerTask<S, V>> for RuntimeBatch<S, V>
{
    fn steal_available_tasks(
        &self,
        worker: &Worker<ManagerTask<S, V>>,
    ) -> Option<ManagerTask<S, V>> {
        loop {
            match self.available_txs.steal_batch_and_pop(worker) {
                Steal::Success(task) => return Some(task),
                Steal::Retry => continue,
                Steal::Empty => return None,
            }
        }
    }

    fn is_depleted(&self) -> bool {
        self.num_pending() == 0 && self.available_txs.is_empty()
    }
}
