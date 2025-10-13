use std::fmt::Debug;

use crate::KVStore;

pub trait WriteableKVStore {
    type Namespace;
    type Error: Debug;
    fn put(&self, ns: Self::Namespace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: Self::Namespace, key: &[u8]) -> Result<(), Self::Error>;
}

impl<T: KVStore> WriteableKVStore for T {
    type Namespace = T::Namespace;
    type Error = T::Error;

    fn put(&self, ns: Self::Namespace, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        KVStore::put(self, ns, key, value)
    }

    fn delete(&self, ns: Self::Namespace, key: &[u8]) -> Result<(), Self::Error> {
        KVStore::delete(self, ns, key)
    }
}
