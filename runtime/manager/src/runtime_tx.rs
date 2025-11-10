use std::sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
};

use kas_l2_core_atomics::AtomicOptionArc;
use kas_l2_core_macros::smart_pointer;
use kas_l2_runtime_execution_workers::Task;
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;

use crate::{
    AccessHandle, ResourceAccess, RuntimeBatchRef, RuntimeManager, StateDiff,
    vm_interface::VmInterface,
};

#[smart_pointer(deref(tx))]
pub struct RuntimeTx<S: Store<StateSpace = StateSpace>, V: VmInterface> {
    vm: V,
    batch: RuntimeBatchRef<S, V>,
    resources: Vec<ResourceAccess<S, V>>,
    pending_resources: AtomicU64,
    effects: AtomicOptionArc<V::TransactionEffects>,
    tx: V::Transaction,
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> RuntimeTx<S, V> {
    pub fn accessed_resources(&self) -> &[ResourceAccess<S, V>] {
        &self.resources
    }

    pub fn effects(&self) -> Option<Arc<V::TransactionEffects>> {
        self.effects.load()
    }

    pub(crate) fn new(
        vm: &V,
        scheduler: &mut RuntimeManager<S, V>,
        state_diffs: &mut Vec<StateDiff<S, V>>,
        batch: RuntimeBatchRef<S, V>,
        tx: V::Transaction,
    ) -> Self {
        Self(Arc::new_cyclic(|this: &Weak<RuntimeTxData<S, V>>| {
            let resources =
                scheduler.resources(&tx, RuntimeTxRef(this.clone()), &batch, state_diffs);
            RuntimeTxData {
                vm: vm.clone(),
                pending_resources: AtomicU64::new(resources.len() as u64),
                effects: AtomicOptionArc::empty(),
                batch,
                tx,
                resources,
            }
        }))
    }

    pub(crate) fn decrease_pending_resources(self) {
        if self.pending_resources.fetch_sub(1, Ordering::Relaxed) == 1 {
            if let Some(batch) = self.batch.upgrade() {
                batch.push_available_tx(&self)
            }
        }
    }

    pub(crate) fn batch(&self) -> &RuntimeBatchRef<S, V> {
        &self.batch
    }
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> RuntimeTxRef<S, V> {
    pub(crate) fn belongs_to_batch(&self, batch: &RuntimeBatchRef<S, V>) -> bool {
        self.upgrade().is_some_and(|tx| tx.batch() == batch)
    }
}

impl<S: Store<StateSpace = StateSpace>, V: VmInterface> Task for RuntimeTx<S, V> {
    fn execute(&self) {
        if let Some(batch) = self.batch.upgrade() {
            let mut handles = self.resources.iter().map(AccessHandle::new).collect::<Vec<_>>();
            match self.vm.process_transaction(&self.tx, &mut handles) {
                Ok(effects) => {
                    self.effects.publish(Arc::new(effects));
                    handles.into_iter().for_each(AccessHandle::commit_changes);
                }
                Err(_) => handles.into_iter().for_each(AccessHandle::rollback_changes),
            }

            batch.decrease_pending_txs();
        }
    }
}
