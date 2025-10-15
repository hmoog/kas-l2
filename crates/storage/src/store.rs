use std::fmt::Debug;

use crate::write::WriteStore;

pub trait Store: Send + Sync + 'static {
    type StateSpace;
    type Error: Debug;
    type WriteBatch: WriteStore<StateSpace = Self::StateSpace, Error = Self::Error>;

    fn get(
        &self,
        state_space: Self::StateSpace,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Self::Error>;
    fn put(
        &self,
        state_space: Self::StateSpace,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Self::Error>;
    fn delete(&self, state_space: Self::StateSpace, key: &[u8]) -> Result<(), Self::Error>;
    fn write_batch(&self) -> Self::WriteBatch;
    fn commit(&self, write_batch: Self::WriteBatch) -> Result<(), Self::Error>;
}
