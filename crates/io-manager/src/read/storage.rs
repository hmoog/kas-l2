use std::fmt::Debug;

use crate::Storage;

pub trait ReadStorage {
    type Namespace;
    type Error: Debug;
    fn get(&self, ns: Self::Namespace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}

impl<T: Storage> ReadStorage for T {
    type Namespace = T::Namespace;
    type Error = T::Error;

    fn get(&self, ns: Self::Namespace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Storage::get(self, ns, key)
    }
}
