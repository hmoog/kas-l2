use kas_l2_storage_store_interface::ReadStore;

pub trait ReadCmd<T>: Send + Sync + 'static {
    fn exec<S: ReadStore<StateSpace = T>>(&self, store: &S);
}
