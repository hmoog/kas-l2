use vprogs_storage_types::Store;

pub trait WriteCmd<T>: Send + Sync + 'static {
    fn exec<S: Store<StateSpace = T>>(&self, store: &S, batch: S::WriteBatch) -> S::WriteBatch;

    fn done(self);
}
