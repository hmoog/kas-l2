use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_runtime_types::Owner;

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, Hash, PartialEq)]
pub struct State<O: Owner> {
    pub owner: O,
    pub data: Vec<u8>,
    pub balance: u64,
}

impl<O: Owner> State<O> {
    pub fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(&self).expect("failed to serialize State")
    }
}

impl<O: Owner> Default for State<O> {
    fn default() -> Self {
        Self { owner: O::default(), data: Vec::new(), balance: 0 }
    }
}

impl<O: Owner> Clone for State<O> {
    fn clone(&self) -> Self {
        Self { owner: self.owner.clone(), data: self.data.clone(), balance: self.balance }
    }
}
