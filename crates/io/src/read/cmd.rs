use crate::ReadStorage;

pub trait ReadCmd<NS>: Send + Sync + 'static {
    fn exec<S: ReadStorage<StateSpace = NS>>(&self, store: &S);
}
