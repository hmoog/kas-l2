use crate::{resources::AccessHandle, transactions::Transaction};

pub trait TransactionProcessor<T: Transaction>:
    Fn(&T, &mut [AccessHandle<T>]) + Clone + Send + Sync + 'static
{
}
impl<T: Transaction, F: Fn(&T, &mut [AccessHandle<T>]) + Clone + Send + Sync + 'static>
    TransactionProcessor<T> for F
{
}
