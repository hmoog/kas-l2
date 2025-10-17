use crate::Store;

pub trait ReadStore {
    type StateSpace;
    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Option<Vec<u8>>;
}

impl<T: Store> ReadStore for T {
    type StateSpace = T::StateSpace;

    fn get(&self, ns: Self::StateSpace, key: &[u8]) -> Option<Vec<u8>> {
        Store::get(self, ns, key)
    }
}
