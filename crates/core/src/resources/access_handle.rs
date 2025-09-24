use std::{ops::Deref, sync::Arc};

use crate::{resources::state::State, transactions::Transaction};

pub enum AccessHandle<T: Transaction> {
    Read(ReadHandle<T>),
    Write(WriteHandle<T>),
}

pub struct ReadHandle<T: Transaction> {
    pub(crate) state: Arc<State<T>>,
    pub(crate) access_metadata: T::AccessMetadata,
}

pub struct WriteHandle<T: Transaction> {
    pub(crate) state: State<T>,
    pub(crate) access_metadata: T::AccessMetadata,
}

impl<T: Transaction> AccessHandle<T> {
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        match self {
            AccessHandle::Read(h) => &h.access_metadata,
            AccessHandle::Write(h) => &h.access_metadata,
        }
    }

    pub fn data(&self) -> &[u8] {
        match self {
            AccessHandle::Read(h) => &h.state.data,
            AccessHandle::Write(h) => &h.state.data,
        }
    }

    pub fn data_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            AccessHandle::Write(h) => Some(&mut h.state.data),
            AccessHandle::Read(_) => None,
        }
    }

    pub fn commit(self) -> Arc<State<T>> {
        match self {
            AccessHandle::Read(h) => h.state,
            AccessHandle::Write(h) => Arc::new(h.state),
        }
    }
}

impl<T: Transaction> Deref for AccessHandle<T> {
    type Target = T::AccessMetadata;

    fn deref(&self) -> &Self::Target {
        self.access_metadata()
    }
}
