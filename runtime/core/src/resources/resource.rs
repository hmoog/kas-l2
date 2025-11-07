use kas_l2_storage_manager::Store;
use tap::Tap;

use crate::{AccessMetadata, BatchRef, ResourceAccess, RuntimeState, RuntimeTxRef, StateDiff, Vm};

pub(crate) struct Resource<S: Store<StateSpace = RuntimeState>, VM: Vm> {
    last_access: Option<ResourceAccess<S, VM>>,
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> Default for Resource<S, VM> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<S: Store<StateSpace = RuntimeState>, VM: Vm> Resource<S, VM> {
    pub(crate) fn access(
        &mut self,
        meta: &VM::AccessMetadata,
        tx: &RuntimeTxRef<S, VM>,
        batch: &BatchRef<S, VM>,
    ) -> ResourceAccess<S, VM> {
        let (state_diff_ref, prev_access) = match self.last_access.take() {
            Some(prev_access) if prev_access.tx().belongs_to_batch(batch) => {
                assert!(prev_access.tx() != tx, "duplicate access to resource");
                (prev_access.state_diff(), Some(prev_access))
            }
            prev_access => (StateDiff::new(batch.clone(), meta.id()), prev_access),
        };

        ResourceAccess::new(meta.clone(), tx.clone(), state_diff_ref, prev_access)
            .tap(|this| self.last_access = Some(this.clone()))
    }
}
