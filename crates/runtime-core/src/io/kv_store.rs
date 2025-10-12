use crate::io::{state_space::StateSpace, write_batch::WriteBatch};

pub trait KVStore: Send + Sync + 'static {
    type WriteBatch<'a>: WriteBatch<'a>;
    type Error;

    fn get(&self, ns: StateSpace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn put(&self, ns: StateSpace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: StateSpace, key: &[u8]) -> Result<(), Self::Error>;
    fn new_write_batch(&self) -> Self::WriteBatch<'_>;
    fn write_batch(&self, batch: Self::WriteBatch<'_>) -> Result<(), Self::Error>;
}
