use std::sync::Arc;
use crate::resources::accessed_resource::AccessedResource;
use crate::resources::state::State;
use crate::{AccessMetadata, AccessType, Transaction};

pub struct ResourceHandle<'a, T: Transaction> {
    state: Arc<State<T>>,
    resource: &'a AccessedResource<T>,
}

impl<'a, T: Transaction> ResourceHandle<'a, T> {
    #[inline]
    pub fn access_metadata(&self) -> &T::Access {
        self.resource
    }

    #[inline]
    pub fn state(&self) -> &State<T> {
        &self.state
    }

    #[inline]
    pub fn state_mut(&mut self) -> &mut State<T> {
        Arc::make_mut(&mut self.state)
    }

    pub(crate) fn new(resource: &'a AccessedResource<T>) -> Self {
        Self { state: resource.read_state(), resource }
    }
}

impl<'a, T: Transaction> Drop for ResourceHandle<'a, T> {
    fn drop(&mut self) {
        if self.resource.access_type() == AccessType::Write {
            self.resource.set_written_state(self.state.clone());
        }
    }
}
