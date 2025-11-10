use std::sync::Arc;

use kas_l2_runtime_interface::{Owner, ResourceId};
use kas_l2_runtime_storage_manager::concat_bytes;
use kas_l2_storage_interface::{ReadStore, WriteStore};
use tap::Tap;

use crate::{
    State, StateSpace,
    StateSpace::{Data, LatestPtr, RollbackPtr},
};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct VersionedState<R: ResourceId, O: Owner> {
    resource_id: R,
    version: u64,
    state: State<O>,
}

impl<R: ResourceId, O: Owner> VersionedState<R, O> {
    pub fn empty(id: R) -> Self {
        Self { resource_id: id, version: 0, state: State::default() }
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
                    state: borsh::from_slice(&data).expect("failed to deserialize State"),
                },
            },
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn state(&self) -> &State<O> {
        &self.state
    }

    pub fn state_mut(self: &mut Arc<Self>) -> &mut State<O> {
        &mut Arc::make_mut(self).tap_mut(|s| s.version += 1).state
    }

    pub fn write_data<S>(&self, store: &mut S)
    where
        S: WriteStore<StateSpace = StateSpace>,
    {
        let key = concat_bytes!(&self.version.to_be_bytes(), &self.resource_id.to_bytes());
        let state_data = self.state.to_bytes();
        store.put(Data, &key, &state_data);
    }

    pub fn write_latest_ptr<S>(&self, store: &mut S)
    where
        S: WriteStore<StateSpace = StateSpace>,
    {
        let key = self.resource_id.to_bytes();
        let version = self.version.to_be_bytes();
        store.put(LatestPtr, &key, &version);
    }

    pub fn write_rollback_ptr<S>(&self, store: &mut S, batch_index: u64)
    where
        S: WriteStore<StateSpace = StateSpace>,
    {
        let key = concat_bytes!(&batch_index.to_be_bytes(), &self.resource_id.to_bytes());
        let version = self.version.to_be_bytes();
        store.put(RollbackPtr, &key, &version);
    }
}

impl<R: ResourceId, O: Owner> Clone for VersionedState<R, O> {
    fn clone(&self) -> Self {
        Self {
            resource_id: self.resource_id.clone(),
            version: self.version,
            state: self.state.clone(),
        }
    }
}
