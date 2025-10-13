use std::fmt::Debug;

pub trait WriteableKVStore {
    type Namespace;
    type Error: Debug;
    fn put(&self, ns: Self::Namespace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: Self::Namespace, key: &[u8]) -> Result<(), Self::Error>;
}
