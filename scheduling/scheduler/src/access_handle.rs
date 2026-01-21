use std::sync::Arc;

use vprogs_core_types::{AccessMetadata, AccessType};
use vprogs_state_space::StateSpace;
use vprogs_state_version::StateVersion;
use vprogs_storage_types::Store;

use crate::{ResourceAccess, vm_interface::VmInterface};

pub struct AccessHandle<'a, S: Store<StateSpace = StateSpace>, V: VmInterface> {
    state_version: Arc<StateVersion<V::ResourceId>>,
    access: &'a ResourceAccess<S, V>,
}

impl<'a, S: Store<StateSpace = StateSpace>, V: VmInterface> AccessHandle<'a, S, V> {
    #[inline]
    pub fn access_metadata(&self) -> &V::AccessMetadata {
        self.access.metadata()
    }

    pub fn version(&self) -> u64 {
        self.state_version.version()
    }

    #[inline]
    pub fn data(&self) -> &Vec<u8> {
        self.state_version.data()
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        self.state_version.data_mut()
    }

    #[inline]
    pub fn is_new(&self) -> bool {
        self.state_version.version() == 0
    }

    pub(crate) fn new(access: &'a ResourceAccess<S, V>) -> Self {
        Self { state_version: access.read_state(), access }
    }

    pub(crate) fn commit_changes(self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.state_version.clone());
        }
    }

    pub(crate) fn rollback_changes(self) {
        if self.access.access_type() == AccessType::Write {
            self.access.set_written_state(self.access.read_state());
        }
    }
}
