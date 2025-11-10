use crate::WriteStore;

pub trait Store: Send + Sync + 'static {
    type StateSpace;
    type WriteBatch: WriteStore<StateSpace = Self::StateSpace>;

    fn get(&self, state_space: Self::StateSpace, key: &[u8]) -> Option<Vec<u8>>;
    fn put(&self, state_space: Self::StateSpace, key: &[u8], value: &[u8]);
    fn delete(&self, state_space: Self::StateSpace, key: &[u8]);
    fn write_batch(&self) -> Self::WriteBatch;
    fn commit(&self, write_batch: Self::WriteBatch);
}
