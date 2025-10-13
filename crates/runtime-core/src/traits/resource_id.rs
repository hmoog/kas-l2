use std::hash::Hash;

use borsh::{BorshDeserialize, BorshSerialize};

pub trait ResourceId:
    BorshSerialize + BorshDeserialize + Default + Eq + Hash + Clone + Send + Sync + 'static
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize(&mut buf)
            .expect("Failed to serialize ResourceId");
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        BorshDeserialize::deserialize_reader(&mut &*bytes)
            .expect("Failed to deserialize ResourceId")
    }
}
impl<T> ResourceId for T where
    T: BorshSerialize + BorshDeserialize + Default + Eq + Hash + Clone + Send + Sync + 'static
{
}
