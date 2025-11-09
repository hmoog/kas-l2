use kas_l2_runtime_interface::AccessMetadata;
use kas_l2_runtime_state_space::StateSpace;
use kas_l2_storage_interface::Store;
use tap::Tap;

use crate::{BatchRef, ResourceAccess, RuntimeTxRef, StateDiff, vm::VM};

pub(crate) struct Resource<S: Store<StateSpace = StateSpace>, V: VM> {
    last_access: Option<ResourceAccess<S, V>>,
}

impl<S: Store<StateSpace = StateSpace>, V: VM> Default for Resource<S, V> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<S: Store<StateSpace = StateSpace>, V: VM> Resource<S, V> {
    pub(crate) fn access(
        &mut self,
        meta: &V::AccessMetadata,
        tx: &RuntimeTxRef<S, V>,
        batch: &BatchRef<S, V>,
    ) -> ResourceAccess<S, V> {
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
