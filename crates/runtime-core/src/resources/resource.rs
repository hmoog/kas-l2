use kas_l2_storage::{Storage, Store};
use tap::Tap;

use crate::{
    AccessMetadata, BatchRef, ResourceAccess, RuntimeState, RuntimeTxRef, StateDiff, Transaction,
    io::{read_cmd::Read, write_cmd::Write},
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
        storage: Storage<S, Read<S, T>, Write<S, T>>,
        meta: &T::AccessMetadata,
        tx: &RuntimeTxRef<S, T>,
        batch: &BatchRef<S, T>,
    ) -> (ResourceAccess<S, T>, Option<StateDiff<S, T>>) {
        let (state_diff_ref, prev_access, new_state_diff) = match self.last_access.take() {
            Some(prev_access) if prev_access.tx().belongs_to_batch(batch) => {
                assert!(prev_access.tx() != tx, "duplicate access to resource");
                (prev_access.state_diff(), Some(prev_access), None)
            }
            prev_access => {
                let new_diff = StateDiff::new(storage, batch.clone(), meta.id());
                (new_diff.downgrade(), prev_access, Some(new_diff))
            }
        };

        let access = ResourceAccess::new(meta.clone(), tx.clone(), state_diff_ref, prev_access)
            .tap(|this| self.last_access = Some(this.clone()));

        (access, new_state_diff)
    }
}
