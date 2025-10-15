use crate::WriteStorage;

pub trait WriteCmd<NS>: Send + Sync + 'static {
    fn exec<S: WriteStorage<StateSpace = NS>>(&self, store: &S);
}
