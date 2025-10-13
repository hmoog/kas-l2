use std::fmt::Debug;

use crate::{ReadableKVStore, WriteableKVStore};

pub trait KVStore: Send + Sync + 'static
where
    Self: WriteableKVStore<Error = <Self as KVStore>::Error, Namespace = <Self as KVStore>::Namespace>
        + ReadableKVStore<Error = <Self as KVStore>::Error, Namespace = <Self as KVStore>::Namespace>,
{
    type Namespace;
    type Error: Debug;
    type WriteBatch: WriteableKVStore<Error = <Self as KVStore>::Error, Namespace = <Self as KVStore>::Namespace>;

    fn new_batch(&self) -> Self::WriteBatch;
    fn write_batch(&self, batch: Self::WriteBatch) -> Result<(), <Self as KVStore>::Error>;
}
