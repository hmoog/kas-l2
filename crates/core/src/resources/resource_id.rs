use std::hash::Hash;

use borsh::{BorshDeserialize, BorshSerialize};

pub trait ResourceID:
    BorshSerialize + BorshDeserialize + Default + Eq + Hash + Clone + Send + Sync + 'static
{
}
impl<T: BorshSerialize + BorshDeserialize + Default + Eq + Hash + Clone + Send + Sync + 'static>
    ResourceID for T
{
}
