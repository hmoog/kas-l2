use crate::WriteStore;

pub trait WriteCmd<T>: Send + Sync + 'static {
    fn exec<S: WriteStore<StateSpace = T>>(&self, store: &S);

    fn commit(self);
}
