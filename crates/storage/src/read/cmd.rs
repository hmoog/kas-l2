use crate::ReadStore;

pub trait ReadCmd<NS>: Send + Sync + 'static {
    fn exec<S: ReadStore<StateSpace = NS>>(&self, store: &S);
}
