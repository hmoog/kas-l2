use crate::WriteStorage;

pub trait WriteCmd<NS>: Send + Sync + 'static {
    fn exec<S: WriteStorage<Namespace = NS>>(&self, store: &S);
}
