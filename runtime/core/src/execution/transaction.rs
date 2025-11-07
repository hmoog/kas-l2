use crate::vm::VM;

pub trait Transaction<V: VM>: Send + Sync + 'static {
    fn accessed_resources(&self) -> &[V::AccessMetadata];
}
