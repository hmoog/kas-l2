use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_storage::concat_bytes;
use tap::Tap;

use crate::{ResourceId, Transaction};

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
        Vec::new().tap_mut(|s| self.serialize(s).expect("serializing state must succeed"))
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
