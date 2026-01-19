use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_transaction_runtime_address::Address;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Data {
    // The program that controls mutations to this object
    owning_program: Address,

    // The type identifier of the object within the program
    type_id: u64,

    // The serialized data of the object
    bytes: Vec<u8>,
}

impl Data {
    pub fn new(owning_program: Address, type_id: u64, bytes: Vec<u8>) -> Self {
        Self { owning_program, type_id, bytes }
    }

    pub fn owning_program(&self) -> &Address {
        &self.owning_program
    }

    pub fn type_id(&self) -> u64 {
        self.type_id
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }
}
