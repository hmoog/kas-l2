use std::{ops::Deref, sync::Arc};

use crate::{
    AccessMetadata, AccessType, Transaction,
    resources::{accessed_resource::AccessedResource, state::State},
};

pub struct ResourceHandle<'a, T: Transaction> {
    state: Arc<State<T>>,
    resource: &'a Arc<AccessedResource<T>>,
}

impl<'a, T: Transaction> ResourceHandle<'a, T> {
    pub(crate) fn new(resource: &'a Arc<AccessedResource<T>>) -> Self {
        Self {
            state: resource.read_state(),
            resource,
        }
    }

    /// Borrow the access metadata.
    pub fn access_metadata(&self) -> &T::Access {
        self.resource
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
}

impl<'a, T: Transaction> Deref for ResourceHandle<'a, T> {
    type Target = T::Access;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'a, T: Transaction> Drop for ResourceHandle<'a, T> {
    fn drop(&mut self) {
        if self.access_type() == AccessType::Write {
            self.resource.set_written_state(self.state.clone());
        }
    }
}
