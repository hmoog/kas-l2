use kas_l2_storage::{ReadStore, WriteStore, concat_bytes};

use crate::{ResourceId, RuntimeState, RuntimeState::Data, Transaction, storage::state::State};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct VersionedState<T: Transaction> {
    pub resource_id: T::ResourceId,
    pub version: u64,
    pub state: State<T>,
}

impl<T: Transaction> VersionedState<T> {
    pub fn from_store<Store: ReadStore<StateSpace = RuntimeState>>(
        store: &Store,
        id: T::ResourceId,
    ) -> Self {
        let id_bytes: Vec<u8> = id.to_bytes();
        match store.get(RuntimeState::LatestPtr, &id_bytes) {
            Some(version) => {
                let Some(data) = store.get(RuntimeState::Data, &concat_bytes!(&version, &id_bytes))
                else {
                    panic!(
                        "data for resource id {:?} with version {:?} not found in storage",
                        id, version
                    );
                };
                Self {
                    resource_id: id,
                    version: u64::from_be_bytes(version[..8].try_into().unwrap()),
                    state: borsh::from_slice(&data).expect("failed to deserialize State"),
                }
            }
            None => Self {
                resource_id: id,
                version: 0,
                state: State::default(),
            },
        }
    }

    pub fn write_data<S: WriteStore<StateSpace = RuntimeState>>(&self, store: &mut S) {
        let key = concat_bytes!(&self.version.to_be_bytes(), &self.resource_id.to_bytes());
        let value = self.state.to_bytes();
        store.put(Data, &key, &value);
    }

    pub fn write_latest_ptr<S: WriteStore<StateSpace = RuntimeState>>(&self, store: &mut S) {
        let key = self.resource_id.to_bytes();
        let value = self.version.to_be_bytes();
        store.put(RuntimeState::LatestPtr, &key, &value);
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
