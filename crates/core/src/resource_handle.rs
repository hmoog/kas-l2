use std::{ops::Deref, sync::Arc};

use crate::{Transaction, resource_state::ResourceState};

pub enum ResourceHandle<T: Transaction> {
    Read {
        state: Arc<ResourceState<T>>,
        access_metadata: T::AccessMetadata,
    },
    Write {
        state: ResourceState<T>,
        access_metadata: T::AccessMetadata,
    },
}

impl<T: Transaction> ResourceHandle<T> {
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        match self {
            ResourceHandle::Read {
                access_metadata, ..
            } => access_metadata,
            ResourceHandle::Write {
                access_metadata, ..
            } => access_metadata,
        }
    }

    pub fn data(&self) -> &[u8] {
        match self {
            ResourceHandle::Read { state, .. } => &state.data,
            ResourceHandle::Write { state, .. } => &state.data,
        }
    }

    pub fn data_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            ResourceHandle::Write { state, .. } => Some(&mut state.data),
            ResourceHandle::Read { .. } => None,
        }
    }

    pub fn commit(self) -> Arc<ResourceState<T>> {
        match self {
            ResourceHandle::Read { state, .. } => state,
            ResourceHandle::Write { state, .. } => Arc::new(state),
        }
    }

    pub fn rollback(self) -> Option<Arc<ResourceState<T>>> {
        match self {
            ResourceHandle::Read { state, .. } => Some(state),
            ResourceHandle::Write { state, .. } => state.prev.and_then(|weak| weak.upgrade()),
        }
    }
}

impl<T: Transaction> Deref for ResourceHandle<T> {
    type Target = T::AccessMetadata;

    fn deref(&self) -> &Self::Target {
        self.access_metadata()
    }
}
