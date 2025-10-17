use borsh::{BorshDeserialize, BorshSerialize};
use tap::Tap;

use crate::Transaction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, Hash, PartialEq)]
pub struct State<T: Transaction> {
    pub version: u64,
    pub owner: T::ResourceId,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<T: Transaction> State<T> {
    pub fn to_bytes(&self) -> Vec<u8> {
        Vec::new().tap_mut(|s| self.serialize(s).expect("serializing state must succeed"))
    }
}

impl<T: Transaction> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            version: self.version,
            owner: self.owner.clone(),
            data: self.data.clone(),
            balance: self.balance,
            executable: self.executable,
        }
    }
}

impl<T: Transaction> Default for State<T> {
    fn default() -> Self {
        Self {
            version: 0,
            owner: T::ResourceId::default(),
            data: Vec::new(),
            balance: 0,
            executable: false,
        }
    }
}
