use tap::Tap;

use crate::{AccessMetadata, BatchApiRef, ResourceAccess, RuntimeTxRef, StateDiff, Transaction};

pub(crate) struct Resource<T: Transaction> {
    last_access: Option<ResourceAccess<T>>,
}

impl<T: Transaction> Default for Resource<T> {
    fn default() -> Self {
        Self { last_access: None }
    }
}

impl<T: Transaction> Resource<T> {
    pub(crate) fn access(
        &mut self,
        meta: &T::AccessMetadata,
        tx: &RuntimeTxRef<T>,
        batch: &BatchApiRef<T>,
    ) -> (ResourceAccess<T>, Option<StateDiff<T>>) {
        let (state_diff_ref, prev_access, new_state_diff) = match self.last_access.take() {
            Some(prev_access) if prev_access.tx().belongs_to_batch(batch) => {
                assert!(prev_access.tx() != tx, "duplicate access to resource");
                (prev_access.state_diff(), Some(prev_access), None)
            }
            prev_access => {
                let new_diff = StateDiff::new(meta.id());
                (new_diff.downgrade(), prev_access, Some(new_diff))
            }
        };

        let access = ResourceAccess::new(meta.clone(), tx.clone(), state_diff_ref, prev_access)
            .tap(|this| self.last_access = Some(this.clone()));

        (access, new_state_diff)
    }
}
