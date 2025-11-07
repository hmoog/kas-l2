use crate::Vm;

pub trait Transaction<VM: Vm>: Send + Sync + 'static {
    fn accessed_resources(&self) -> &[VM::AccessMetadata];
}
