use std::sync::Arc;

use kas_l2_storage_manager::Store;

use crate::{
    AccessMetadata, AccessType, ResourceAccess, RuntimeState, State, VersionedState, vm::VM,
};

pub struct AccessHandle<'a, S: Store<StateSpace = RuntimeState>, V: VM> {
    versioned_state: Arc<VersionedState<V>>,
    access: &'a ResourceAccess<S, V>,
}

impl<'a, S: Store<StateSpace = RuntimeState>, V: VM> AccessHandle<'a, S, V> {
    #[inline]
    pub fn access_metadata(&self) -> &V::AccessMetadata {
        self.access.metadata()
    }

    pub fn version(&self) -> u64 {
        self.versioned_state.version()
    }

    #[inline]
    pub fn state(&self) -> &State<V> {
        self.versioned_state.state()
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut State<V> {
        self.versioned_state.state_mut()
    }

    #[inline]
    pub fn is_new(&self) -> bool {
        self.versioned_state.version() == 0
    }

    pub(crate) fn new(access: &'a ResourceAccess<S, V>) -> Self {
        Self { versioned_state: access.read_state(), access }
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
