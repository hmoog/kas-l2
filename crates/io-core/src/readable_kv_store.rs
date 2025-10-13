use std::fmt::Debug;

pub trait ReadableKVStore {
    type Namespace;
    type Error: Debug;
    fn get(&self, ns: Self::Namespace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
}
