use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_storage::{ReadStore, concat_bytes};

use crate::{ResourceId, RuntimeState, Transaction};

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, Hash, PartialEq)]
pub struct State<T: Transaction> {
    pub resource_id: T::ResourceId,
    pub version: u64,
    pub owner: T::ResourceId,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<T: Transaction> State<T> {
    pub(crate) fn new(resource_id: T::ResourceId, version: u64) -> Self {
        Self {
            resource_id,
            version,
            owner: T::ResourceId::default(),
            data: Vec::new(),
            balance: 0,
            executable: false,
        }
    }

    pub fn versioned_id(&self) -> Vec<u8> {
        concat_bytes!(&self.version.to_be_bytes(), &self.resource_id.to_bytes())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("failed to serialize State")
    }

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
                borsh::from_slice(&data).expect("failed to deserialize State")
            }
            None => State::new(id.clone(), 0),
        }
    }
}

impl<T: Transaction> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            resource_id: self.resource_id.clone(),
            version: self.version,
            owner: self.owner.clone(),
            data: self.data.clone(),
            balance: self.balance,
            executable: self.executable,
        }
    }
}
