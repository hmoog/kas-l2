use crate::{Transaction, resources::resource_handle::ResourceHandle};

pub trait TransactionProcessor<T: Transaction>:
    Fn(&T, &mut [ResourceHandle<T>]) + Clone + Send + Sync + 'static
{
}
impl<T, F> TransactionProcessor<T> for F
where
    T: Transaction,
    F: Fn(&T, &mut [ResourceHandle<T>]) + Clone + Send + Sync + 'static,
{
}
