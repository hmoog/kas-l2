use crate::Store;

pub trait WriteStore {
    type StateSpace;
    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]);
    fn delete(&self, ns: Self::StateSpace, key: &[u8]);
}

impl<T: Store> WriteStore for T {
    type StateSpace = T::StateSpace;

    fn put(&self, ns: Self::StateSpace, key: &[u8], value: &[u8]) {
        Store::put(self, ns, key, value)
    }

    fn delete(&self, ns: Self::StateSpace, key: &[u8]) {
        Store::delete(self, ns, key)
    }
}
