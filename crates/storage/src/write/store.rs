use std::fmt::Debug;

use crate::Store;

pub trait WriteStore {
    type StateSpace;
    type Error: Debug;
    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: Self::StateSpace, key: &[u8]) -> Result<(), Self::Error>;
}

impl<T: Store> WriteStore for T {
    type StateSpace = T::StateSpace;
    type Error = T::Error;

    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        Store::put(self, ns, key, value)
    }

    fn delete(&self, ns: Self::StateSpace, key: &[u8]) -> Result<(), Self::Error> {
        Store::delete(self, ns, key)
    }
}
