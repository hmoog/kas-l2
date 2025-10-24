use std::sync::Arc;

use kas_l2_storage::Store;

use crate::{
    AccessMetadata, AccessType, ResourceAccess, RuntimeState, State, Transaction, VersionedState,
};

pub struct AccessHandle<'a, S: Store<StateSpace = RuntimeState>, T: Transaction> {
    versioned_state: Arc<VersionedState<T>>,
    access: &'a ResourceAccess<S, T>,
}

impl<'a, S: Store<StateSpace = RuntimeState>, T: Transaction> AccessHandle<'a, S, T> {
    #[inline]
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        self.access.metadata()
    }

    #[inline]
    pub fn state(&self) -> &State<T> {
        self.versioned_state.state()
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut State<T> {
        self.versioned_state.state_mut()
    }

    pub(crate) fn new(access: &'a ResourceAccess<S, T>) -> Self {
        Self {
            versioned_state: access.read_state(),
            access,
        }
    }

    pub(crate) fn commit_changes(self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.versioned_state.clone());
        }
    }

    pub(crate) fn rollback_changes(self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.access.read_state());
        }
    }
}
