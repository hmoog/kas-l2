use std::fmt::Debug;

use crate::Storage;

pub trait ReadStorage {
    type StateSpace;
    type Error: Debug;
    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}

impl<T: Storage> ReadStorage for T {
    type StateSpace = T::StateSpace;
    type Error = T::Error;

    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Storage::get(self, ns, key)
    }
}
