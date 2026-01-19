use borsh::{BorshDeserialize, BorshSerialize};
use vprogs_transaction_runtime_address::Address;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Default, BorshDeserialize, BorshSerialize)]
pub enum ObjectId {
    #[default]
    Empty,
    Program(Address),
    Data(Address),
}
