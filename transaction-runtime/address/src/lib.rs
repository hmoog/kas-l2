use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, Eq, Hash, PartialEq, BorshSerialize, BorshDeserialize, Copy)]
pub struct Address([u8; 32]);

impl Address {
    pub const SYSTEM: Address = Address([0u8; 32]);
}
