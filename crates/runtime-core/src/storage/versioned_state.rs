use kas_l2_storage::{ReadStore, WriteStore, concat_bytes};

use crate::{
    ResourceId, RuntimeState,
    RuntimeState::{Data, LatestPtr, RollbackPtr},
    Transaction,
    storage::state::State,
};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct VersionedState<T: Transaction> {
    pub resource_id: T::ResourceId,
    pub version: u64,
    pub state: State<T>,
}

impl<T: Transaction> VersionedState<T> {
    pub fn empty(id: T::ResourceId) -> Self {
        Self {
            resource_id: id,
            version: 0,
            state: State::default(),
        }
    }

    pub fn from_store<S>(store: &S, id: T::ResourceId) -> Self
    where
        S: ReadStore<StateSpace = RuntimeState>,
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

    pub(crate) fn write_data<S>(&self, store: &mut S)
    where
        S: WriteStore<StateSpace = RuntimeState>,
    {
        let key = concat_bytes!(&self.version.to_be_bytes(), &self.resource_id.to_bytes());
        let state_data = self.state.to_bytes();
        store.put(Data, &key, &state_data);
    }

    pub(crate) fn write_latest_ptr<S>(&self, store: &mut S)
    where
        S: WriteStore<StateSpace = RuntimeState>,
    {
        let key = self.resource_id.to_bytes();
        let version = self.version.to_be_bytes();
        store.put(LatestPtr, &key, &version);
    }

    pub(crate) fn write_rollback_ptr<S>(&self, store: &mut S, batch_index: u64)
    where
        S: WriteStore<StateSpace = RuntimeState>,
    {
        let key = concat_bytes!(&batch_index.to_be_bytes(), &self.resource_id.to_bytes());
        let version = self.version.to_be_bytes();
        store.put(RollbackPtr, &key, &version);
    }
}

impl<T: Transaction> Clone for VersionedState<T> {
    fn clone(&self) -> Self {
        Self {
            resource_id: self.resource_id.clone(),
            version: self.version,
            state: self.state.clone(),
        }
    }
}
