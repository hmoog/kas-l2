use kas_l2_storage::Store;
use tap::Tap;

use crate::{
    AccessMetadata, BatchRef, ResourceAccess, RuntimeState, RuntimeTxRef, StateDiff, Transaction,
};

pub(crate) struct Resource<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    last_access: Option<ResourceAccess<S, T>>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> Default for Resource<S, T> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> Resource<S, T> {
    pub(crate) fn access(
        &mut self,
        meta: &T::AccessMetadata,
        tx: &RuntimeTxRef<S, T>,
        batch: &BatchRef<S, T>,
    ) -> ResourceAccess<S, T> {
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
