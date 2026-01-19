pub trait WriteBatch {
    type StateSpace;
    fn put(&mut self, ns: Self::StateSpace, key: &[u8], value: &[u8]);
    fn delete(&mut self, ns: Self::StateSpace, key: &[u8]);
}
