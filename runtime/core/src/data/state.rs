use borsh::{BorshDeserialize, BorshSerialize};

use crate::Vm;

#[derive(BorshSerialize, BorshDeserialize, Debug, Eq, Hash, PartialEq)]
pub struct State<VM: Vm> {
    pub owner: VM::ResourceId,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<VM: Vm> State<VM> {
    pub fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(&self).expect("failed to serialize State")
    }
}

impl<VM: Vm> Default for State<VM> {
    fn default() -> Self {
        Self { owner: VM::ResourceId::default(), data: Vec::new(), balance: 0, executable: false }
    }
}

impl<VM: Vm> Clone for State<VM> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            data: self.data.clone(),
            balance: self.balance,
            executable: self.executable,
        }
    }
}
