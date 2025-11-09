use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_core_macros::smart_pointer;
use kas_l2_runtime_state_space::StateSpace;
use kas_l2_storage_interface::Store;

use kas_l2_runtime_execution::ExecutionTask;

use crate::{
    AccessHandle, BatchRef, ResourceAccess, StateDiff, scheduling::scheduler::Scheduler, vm::VM,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<S: Store<StateSpace = StateSpace>, V: VM> {
    batch: BatchRef<S, V>,
    resources: Vec<ResourceAccess<S, V>>,
    pending_resources: AtomicU64,
    tx: V::Transaction,
}

impl<S: Store<StateSpace = StateSpace>, V: VM> RuntimeTx<S, V> {
    pub fn accessed_resources(&self) -> &[ResourceAccess<S, V>] {
        &self.resources
    }

    pub(crate) fn new(
        scheduler: &mut Scheduler<S, V>,
        state_diffs: &mut Vec<StateDiff<S, V>>,
        batch: BatchRef<S, V>,
        tx: V::Transaction,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<S, V>>| {
            let resources =
                scheduler.resources(&tx, RuntimeTxRef(this.clone()), &batch, state_diffs);
            RuntimeTxData {
                pending_resources: AtomicU64::new(resources.len() as u64),
                batch,
                tx,
                resources,
            }
        }))
    }

    pub(crate) fn execute(&self, vm: &V) {
        if let Some(batch) = self.batch.upgrade() {
            let mut handles = self.resources.iter().map(AccessHandle::new).collect::<Vec<_>>();
            match vm.process_transaction(&self.tx, &mut handles) {
                Ok(()) => handles.into_iter().for_each(AccessHandle::commit_changes),
                Err(_) => handles.into_iter().for_each(AccessHandle::rollback_changes),
            }

            batch.decrease_pending_txs();
        }
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.pending_resources.fetch_sub(1, Ordering::Relaxed) == 1 {
            if let Some(batch) = self.batch.upgrade() {
                batch.push_available_tx(&self)
            }
        }
    }

    pub(crate) fn batch(&self) -> &BatchRef<S, V> {
        &self.batch
    }
}

impl<S: Store<StateSpace = StateSpace>, V: VM> RuntimeTxRef<S, V> {
    pub(crate) fn belongs_to_batch(&self, batch: &BatchRef<S, V>) -> bool {
        self.upgrade().is_some_and(|tx| tx.batch() == batch)
    }
}

impl<S, V> ExecutionTask<V> for RuntimeTx<S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VM,
{
    fn execute_with(&self, vm: &V) {
        RuntimeTx::execute(self, vm);
    }
}
