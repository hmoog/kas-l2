use crate::ReadStorage;

pub trait ReadCmd<NS>: Send + Sync + 'static {
    fn exec<S: ReadStorage<Namespace = NS>>(&self, store: &S);
}
