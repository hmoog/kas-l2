use crate::WriteStore;

pub trait WriteCmd<T>: Send + Sync + 'static {
    fn exec<S: WriteStore<StateSpace = T>>(&self, store: &mut S);

    fn mark_committed(self);
}
