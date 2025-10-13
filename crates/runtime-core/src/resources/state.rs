use borsh::{BorshDeserialize, BorshSerialize};

use crate::Transaction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, Hash, PartialEq)]
pub struct State<T: Transaction> {
    pub owner: T::ResourceId,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<T: Transaction> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
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
            owner: T::ResourceId::default(),
            data: Vec::new(),
            balance: 0,
            executable: false,
        }
    }
}
