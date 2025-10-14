use std::fmt::Debug;

use crate::Storage;

pub trait WriteStorage {
    type Namespace;
    type Error: Debug;
    fn put(&self, ns: Self::Namespace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: Self::Namespace, key: &[u8]) -> Result<(), Self::Error>;
}

impl<T: Storage> WriteStorage for T {
    type Namespace = T::Namespace;
    type Error = T::Error;

    fn put(&self, ns: Self::Namespace, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        Storage::put(self, ns, key, value)
    }

    fn delete(&self, ns: Self::Namespace, key: &[u8]) -> Result<(), Self::Error> {
        Storage::delete(self, ns, key)
    }
}
