use crate::WriteStore;

pub trait WriteCmd<NS>: Send + Sync + 'static {
    fn exec<S: WriteStore<StateSpace = NS>>(&self, store: &S);
}
