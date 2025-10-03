use std::sync::Arc;

use crate::{AccessMetadata, AccessType, ResourceAccess, State, Transaction};

pub struct AccessHandle<'a, T: Transaction> {
    state: Arc<State<T>>,
    access: &'a ResourceAccess<T>,
}

impl<'a, T: Transaction> AccessHandle<'a, T> {
    #[inline]
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        self.access.metadata()
    }

    #[inline]
    pub fn state(&self) -> &State<T> {
        &self.state
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut State<T> {
        Arc::make_mut(&mut self.state)
    }

    pub(crate) fn new(access: &'a ResourceAccess<T>) -> Self {
        Self {
            state: access.read_state(),
            access,
        }
    }

    pub(crate) fn commit_changes(self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.state.clone());
        }
    }

    pub(crate) fn rollback_changes(self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.access.read_state());
        }
    }
}
