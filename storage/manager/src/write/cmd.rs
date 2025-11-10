use kas_l2_storage_types::WriteStore;

pub trait WriteCmd<T>: Send + Sync + 'static {
    fn exec<S: WriteStore<StateSpace = T>>(&self, store: &mut S);

    fn done(self);
}
