use std::sync::Arc;

use tap::Tap;
use vprogs_scheduling_types::ResourceId;
use vprogs_state_space::{
    StateSpace,
    StateSpace::{Data, LatestPtr, RollbackPtr},
};
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
        let id_bytes: Vec<u8> = id.to_bytes();
        match store.get(LatestPtr, &id_bytes) {
            None => Self::empty(id),
            Some(version) => match store.get(Data, &concat_bytes!(&version, &id_bytes)) {
                None => panic!("missing data for resource_{:?}@v{:?}", id, version),
                Some(data) => Self {
                    resource_id: id,
                    version: u64::from_be_bytes(version[..8].try_into().unwrap()),
                    data,
                },
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
        let key = concat_bytes!(&self.version.to_be_bytes(), &self.resource_id.to_bytes());
        store.put(Data, &key, &self.data);
    }

    pub fn write_latest_ptr<W>(&self, store: &mut W)
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        let key = self.resource_id.to_bytes();
        let version = self.version.to_be_bytes();
        store.put(LatestPtr, &key, &version);
    }

    pub fn write_rollback_ptr<W>(&self, store: &mut W, batch_index: u64)
    where
        W: WriteBatch<StateSpace = StateSpace>,
    {
        let key = concat_bytes!(&batch_index.to_be_bytes(), &self.resource_id.to_bytes());
        let version = self.version.to_be_bytes();
        store.put(RollbackPtr, &key, &version);
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
