use std::sync::Arc;

use kas_l2_runtime_interface::{AccessMetadata, AccessType};
use kas_l2_runtime_state::{State, StateSpace, VersionedState};
use kas_l2_storage_interface::Store;

use crate::{ResourceAccess, vm::VmInterface};

pub struct AccessHandle<'a, S: Store<StateSpace = StateSpace>, V: VmInterface> {
    versioned_state: Arc<VersionedState<V::ResourceId, V::Ownership>>,
    access: &'a ResourceAccess<S, V>,
}

impl<'a, S: Store<StateSpace = StateSpace>, V: VmInterface> AccessHandle<'a, S, V> {
    #[inline]
    pub fn access_metadata(&self) -> &V::AccessMetadata {
        self.access.metadata()
    }

    pub fn version(&self) -> u64 {
        self.versioned_state.version()
    }

    #[inline]
    pub fn state(&self) -> &State<V::Ownership> {
        self.versioned_state.state()
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut State<V::Ownership> {
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
