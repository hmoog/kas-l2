use std::sync::Arc;

use crate::{AccessMetadata, AccessType, ResourceAccess, State, Transaction};

pub struct AccessHandle<'a, T: Transaction> {
    state: Arc<State<T>>,
    access: &'a ResourceAccess<T>,
}

impl<'a, T: Transaction> AccessHandle<'a, T> {
    #[inline]
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        self.access
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
}

impl<'a, T: Transaction> Drop for AccessHandle<'a, T> {
    fn drop(&mut self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.state.clone());
        }
    }
}
