use crate::{Transaction, resources::resource_handle::ResourceHandle};

pub trait TransactionProcessor<T: Transaction>:
    Fn(&T, &mut [ResourceHandle<T>]) + Clone + Send + Sync + 'static
{
}
impl<T: Transaction, F: Fn(&T, &mut [ResourceHandle<T>]) + Clone + Send + Sync + 'static>
    TransactionProcessor<T> for F
{
}
