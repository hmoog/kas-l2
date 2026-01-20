use std::{fmt::Debug, hash::Hash};

use borsh::{BorshDeserialize, BorshSerialize};

pub trait ResourceId:
    BorshSerialize + BorshDeserialize + Debug + Default + Eq + Hash + Clone + Send + Sync + 'static
{
    fn to_bytes(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("failed to serialize ResourceId")
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        borsh::from_slice(bytes).expect("failed to deserialize ResourceId")
    }
}
impl<T> ResourceId for T where
    T: BorshSerialize
        + BorshDeserialize
        + Debug
        + Default
        + Eq
        + Hash
        + Clone
        + Send
        + Sync
        + 'static
{
}
