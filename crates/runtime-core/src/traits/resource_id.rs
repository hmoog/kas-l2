use std::hash::Hash;

use borsh::{BorshDeserialize, BorshSerialize};

pub trait ResourceId:
    BorshSerialize + BorshDeserialize + Default + Eq + Hash + Clone + Send + Sync + 'static
{
}
impl<T> ResourceId for T where
    T: BorshSerialize + BorshDeserialize + Default + Eq + Hash + Clone + Send + Sync + 'static
{
}
