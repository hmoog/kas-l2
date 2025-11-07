use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_core_macros::smart_pointer;
use kas_l2_storage_manager::Store;

use crate::{
    AccessHandle, BatchRef, ResourceAccess, RuntimeState, StateDiff, VecExt, Vm,
    scheduling::scheduler::Scheduler,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    batch: BatchRef<S, VM>,
    resources: Vec<ResourceAccess<S, VM>>,
    pending_resources: AtomicU64,
    tx: VM::Transaction,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> RuntimeTx<S, VM> {
    pub fn accessed_resources(&self) -> &[ResourceAccess<S, VM>] {
        &self.resources
    }

    pub(crate) fn new(
        scheduler: &mut Scheduler<S, VM>,
        state_diffs: &mut Vec<StateDiff<S, VM>>,
        batch: BatchRef<S, VM>,
        tx: VM::Transaction,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<S, VM>>| {
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

    pub(crate) fn execute(&self, vm: &VM) {
        if let Some(batch) = self.batch.upgrade() {
            let mut handles = self.resources.as_vec(AccessHandle::new);
            match vm.process(&self.tx, &mut handles) {
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

    pub(crate) fn batch(&self) -> &BatchRef<S, Tx> {
        &self.batch
    }
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> RuntimeTxRef<S, VM> {
    pub(crate) fn belongs_to_batch(&self, batch: &BatchRef<S, VM>) -> bool {
        self.upgrade().is_some_and(|tx| tx.batch() == batch)
    }
}
