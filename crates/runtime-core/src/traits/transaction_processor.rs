use crate::{AccessHandle, Transaction};

pub trait TransactionProcessor<T: Transaction>:
    Fn(&T, &mut [AccessHandle<T>]) + Clone + Send + Sync + 'static
{
}
impl<T, F> TransactionProcessor<T> for F
where
    T: Transaction,
    F: Fn(&T, &mut [AccessHandle<T>]) + Clone + Send + Sync + 'static,
{
}
