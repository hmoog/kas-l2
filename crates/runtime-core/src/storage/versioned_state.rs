use kas_l2_storage::{ReadStore, concat_bytes};

use crate::{ResourceId, RuntimeState, Transaction, storage::state::State};

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
        match store.get(RuntimeState::DataPointers, &id_bytes) {
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

    pub fn id(&self) -> Vec<u8> {
        concat_bytes!(&self.version.to_be_bytes(), &self.resource_id.to_bytes())
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
