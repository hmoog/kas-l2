use std::fmt::Debug;

use crate::Storage;

pub trait WriteStorage {
    type StateSpace;
    type Error: Debug;
    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: Self::StateSpace, key: &[u8]) -> Result<(), Self::Error>;
}

impl<T: Storage> WriteStorage for T {
    type StateSpace = T::StateSpace;
    type Error = T::Error;

    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        Storage::put(self, ns, key, value)
    }

    fn delete(&self, ns: Self::StateSpace, key: &[u8]) -> Result<(), Self::Error> {
        Storage::delete(self, ns, key)
    }
}
