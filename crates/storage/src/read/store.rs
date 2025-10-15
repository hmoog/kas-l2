use std::fmt::Debug;

use crate::Store;

pub trait ReadStore {
    type StateSpace;
    type Error: Debug;
    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}

impl<T: Store> ReadStore for T {
    type StateSpace = T::StateSpace;
    type Error = T::Error;

    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Store::get(self, ns, key)
    }
}
