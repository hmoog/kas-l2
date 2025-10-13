use kas_l2_io_core::ReadableKVStore;

pub trait Cmd<NS>: Send + Sync + 'static {
    fn exec<S: ReadableKVStore<Namespace = NS>>(&self, store: &S);
}
