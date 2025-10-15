use std::fmt::Debug;

use crate::write::WriteStorage;

pub trait Storage: Send + Sync + 'static {
    type StateSpace;
    type Error: Debug;
    type WriteBatch: WriteStorage<StateSpace = Self::StateSpace, Error = Self::Error>;

    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&self, ns: Self::StateSpace, key: &[u8]) -> Result<(), Self::Error>;
    fn new_batch(&self) -> Self::WriteBatch;
    fn write_batch(&self, batch: Self::WriteBatch) -> Result<(), Self::Error>;
}
