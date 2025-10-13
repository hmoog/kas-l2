use kas_l2_io_core::WriteableKVStore;

pub trait Cmd<NS>: Send + Sync + 'static {
    fn exec<S: WriteableKVStore<Namespace = NS>>(&self, store: &S);
}
