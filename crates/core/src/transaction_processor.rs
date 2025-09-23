use crate::{ResourceHandle, Transaction};

pub trait TransactionProcessor<T: Transaction>:
    Fn(&T, &[ResourceHandle<T::ResourceID, T::AccessMetadata>]) + Clone + Send + Sync + 'static
{
}
impl<
    T: Transaction,
    F: Fn(&T, &[ResourceHandle<T::ResourceID, T::AccessMetadata>]) + Clone + Send + Sync + 'static,
> TransactionProcessor<T> for F
{
}
