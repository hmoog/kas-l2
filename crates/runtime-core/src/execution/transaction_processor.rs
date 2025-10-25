use kas_l2_storage::Store;

use crate::{AccessHandle, RuntimeState, Transaction};

pub trait TransactionProcessor<S: Store<StateSpace = RuntimeState>, Tx>:
    Fn(&Tx, &mut [AccessHandle<S, Tx>]) -> Result<(), Self::Error> + Clone + Send + Sync + 'static
where
    Tx: Transaction,
{
    type Error;
}

impl<S: Store<StateSpace = RuntimeState>, Tx, Err, T> TransactionProcessor<S, Tx> for T
where
    Tx: Transaction,
    T: Fn(&Tx, &mut [AccessHandle<S, Tx>]) -> Result<(), Err> + Clone + Send + Sync + 'static,
{
    type Error = Err;
}
