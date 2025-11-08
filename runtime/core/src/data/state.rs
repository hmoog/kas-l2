use borsh::{BorshDeserialize, BorshSerialize};

use crate::vm::VM;

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, Hash, PartialEq)]
pub struct State<V: VM> {
    pub owner: V::Ownership,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<V: VM> State<V> {
    pub fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(&self).expect("failed to serialize State")
    }
}

impl<V: VM> Default for State<V> {
    fn default() -> Self {
        Self { owner: V::Ownership::default(), data: Vec::new(), balance: 0, executable: false }
    }
}

impl<V: VM> Clone for State<V> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            data: self.data.clone(),
            balance: self.balance,
            executable: self.executable,
        }
    }
}
