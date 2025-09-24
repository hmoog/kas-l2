use std::{ops::Deref, sync::Arc};

use crate::{Transaction, resources::resource_state::ResourceState};

pub enum ResourceHandle<T: Transaction> {
    Read(ReadHandle<T>),
    Write(WriteHandle<T>),
}

pub struct ReadHandle<T: Transaction> {
    pub(crate) state: Arc<ResourceState<T>>,
    pub(crate) access_metadata: T::AccessMetadata,
}

pub struct WriteHandle<T: Transaction> {
    pub(crate) state: ResourceState<T>,
    pub(crate) access_metadata: T::AccessMetadata,
}

impl<T: Transaction> ResourceHandle<T> {
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        match self {
            ResourceHandle::Read(h) => &h.access_metadata,
            ResourceHandle::Write(h) => &h.access_metadata,
        }
    }

    pub fn data(&self) -> &[u8] {
        match self {
            ResourceHandle::Read(h) => &h.state.data,
            ResourceHandle::Write(h) => &h.state.data,
        }
    }

    pub fn data_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            ResourceHandle::Write(h) => Some(&mut h.state.data),
            ResourceHandle::Read(_) => None,
        }
    }

    pub fn commit(self) -> Arc<ResourceState<T>> {
        match self {
            ResourceHandle::Read(h) => h.state,
            ResourceHandle::Write(h) => Arc::new(h.state),
        }
    }
}

impl<T: Transaction> Deref for ResourceHandle<T> {
    type Target = T::AccessMetadata;

    fn deref(&self) -> &Self::Target {
        self.access_metadata()
    }
}
