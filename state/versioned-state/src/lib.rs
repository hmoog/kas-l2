use std::sync::Arc;

use tap::Tap;
use vprogs_core_types::ResourceId;
use vprogs_state_latest_ptr::LatestPtr;
use vprogs_state_rollback_ptr::RollbackPtr;
use vprogs_state_space::StateSpace;
use vprogs_storage_manager::concat_bytes;
use vprogs_storage_types::{ReadStore, WriteBatch};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct VersionedState<R: ResourceId> {
    resource_id: R,
    version: u64,
    data: Vec<u8>,
}

impl<R: ResourceId> VersionedState<R> {
    pub fn empty(id: R) -> Self {
        Self { resource_id: id, version: 0, data: Vec::new() }
    }

    pub fn from_latest_data<S>(store: &S, id: R) -> Self
    where
        S: ReadStore<StateSpace = StateSpace>,
    {
        match LatestPtr::get(store, &id) {
            None => Self::empty(id),
            Some(version) => match Self::get(store, version, &id) {
                None => panic!("missing data for resource_{:?}@v{:?}", id, version),
                Some(data) => Self { resource_id: id, version, data },
            },
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn data_mut(self: &mut Arc<Self>) -> &mut Vec<u8> {
        &mut Arc::make_mut(self).tap_mut(|s| s.version += 1).data
    }

    pub fn write_data<W>(&self, store: &mut W)
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        Self::put(store, self.version, &self.resource_id, &self.data);
    }

    pub fn write_latest_ptr<W>(&self, store: &mut W)
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        LatestPtr::put(store, &self.resource_id, self.version);
    }

    pub fn write_rollback_ptr<W>(&self, store: &mut W, batch_index: u64)
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        RollbackPtr::put(store, batch_index, &self.resource_id, self.version);
    }

    /// Gets the data for a specific version of a resource.
    ///
    /// Key layout: `version.to_be_bytes() || resource_id.to_bytes()`
    pub fn get<S>(store: &S, version: u64, resource_id: &R) -> Option<Vec<u8>>
    where
        S: ReadStore<StateSpace = StateSpace>,
    {
        let key = concat_bytes!(&version.to_be_bytes(), &resource_id.to_bytes());
        store.get(StateSpace::Data, &key)
    }

    /// Stores data for a specific version of a resource.
    ///
    /// Key layout: `version.to_be_bytes() || resource_id.to_bytes()`
    pub fn put<W>(store: &mut W, version: u64, resource_id: &R, data: &[u8])
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        let key = concat_bytes!(&version.to_be_bytes(), &resource_id.to_bytes());
        store.put(StateSpace::Data, &key, data);
    }

    /// Deletes data for a specific version of a resource.
    ///
    /// Key layout: `version.to_be_bytes() || resource_id.to_bytes()`
    pub fn delete<W>(store: &mut W, version: u64, resource_id: &R)
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        let key = concat_bytes!(&version.to_be_bytes(), &resource_id.to_bytes());
        store.delete(StateSpace::Data, &key);
    }
}

impl<R: ResourceId> Clone for VersionedState<R> {
    fn clone(&self) -> Self {
        Self {
            resource_id: self.resource_id.clone(),
            version: self.version,
            data: self.data.clone(),
        }
    }
}
