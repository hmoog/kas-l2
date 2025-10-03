use crate::{AccessHandle, Transaction};

pub trait TransactionProcessor<Tx>:
    Fn(&Tx, &mut [AccessHandle<Tx>]) -> Result<(), Self::Error> + Clone + Send + Sync + 'static
where
    Tx: Transaction,
{
    type Error;
}

impl<Tx, Err, T> TransactionProcessor<Tx> for T
where
    Tx: Transaction,
    T: Fn(&Tx, &mut [AccessHandle<Tx>]) -> Result<(), Err> + Clone + Send + Sync + 'static,
{
    type Error = Err;
}
