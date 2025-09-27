use borsh::{BorshDeserialize, BorshSerialize};

use crate::transactions::Transaction;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct State<T: Transaction> {
    pub owner: T::ResourceID,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<T: Transaction> State<T> {
    pub fn new(owner: T::ResourceID, data: Vec<u8>, balance: u64, executable: bool) -> Self {
        Self {
            owner,
            data,
            balance,
            executable,
        }
    }
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
            owner: T::ResourceID::default(),
            data: Vec::new(),
            balance: 0,
            executable: false,
        }
    }
}
