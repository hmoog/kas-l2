use vprogs_storage_types::ReadStore;

pub trait ReadCmd<T>: Send + Sync + 'static {
    fn exec<S: ReadStore<StateSpace = T>>(&self, store: &S);
}
