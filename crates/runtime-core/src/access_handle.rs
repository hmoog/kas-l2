use std::{ops::Deref, sync::Arc};

use crate::{State, Transaction};

pub struct AccessHandle<'a, T: Transaction> {
    state: Arc<State<T>>,
    metadata: &'a T::AccessMetadata,
}

impl<'a, T: Transaction> AccessHandle<'a, T> {
    pub fn new(state: Arc<State<T>>, metadata: &'a T::AccessMetadata) -> Self {
        Self { state, metadata }
    }

    /// Borrow the access metadata.
    pub fn access_metadata(&self) -> &T::AccessMetadata {
        self.metadata
    }

    /// Immutable access to the underlying data.
    pub fn data(&self) -> &[u8] {
        &self.state.data
    }

    /// Mutable access with copy-on-write semantics.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut Arc::make_mut(&mut self.state).data
    }

    /// Ensure the Vec has at least `additional` capacity.
    pub fn reserve(&mut self, additional: usize) {
        Arc::make_mut(&mut self.state).data.reserve(additional);
    }

    /// Replace the underlying Vec entirely.
    pub fn set_data(&mut self, new_data: Vec<u8>) {
        Arc::make_mut(&mut self.state).data = new_data;
    }

    /// Resize the underlying Vec (zero-filling new elements).
    pub fn resize(&mut self, new_len: usize) {
        Arc::make_mut(&mut self.state).data.resize(new_len, 0);
    }

    /// Commit and return the underlying Arc<State<T>>.
    pub fn commit(self) -> Arc<State<T>> {
        self.state
    }
}

impl<'a, T: Transaction> Deref for AccessHandle<'a, T> {
    type Target = T::AccessMetadata;

    fn deref(&self) -> &Self::Target {
        self.metadata
    }
}
