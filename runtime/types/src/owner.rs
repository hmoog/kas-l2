use std::{fmt::Debug, hash::Hash};

use borsh::{BorshDeserialize, BorshSerialize};

pub trait Owner:
    Debug + Default + Eq + Hash + Clone + BorshSerialize + BorshDeserialize + Send + Sync + 'static
{
}

impl<T> Owner for T where
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
