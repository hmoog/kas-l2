use std::fmt::Debug;

use crate::KVStore;

pub trait ReadableKVStore {
    type Namespace;
    type Error: Debug;
    fn get(&self, ns: Self::Namespace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}

impl<T: KVStore> ReadableKVStore for T {
    type Namespace = T::Namespace;
    type Error = T::Error;

    fn get(&self, ns: Self::Namespace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        KVStore::get(self, ns, key)
    }
}
