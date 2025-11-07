use kas_l2_storage_manager::Store;

use crate::{AccessHandle, RuntimeState, vm::VM};

pub trait TransactionProcessor<S: Store<StateSpace = RuntimeState>, V: VM>:
    Fn(&V::Transaction, &mut [AccessHandle<S, V>]) -> Result<(), Self::Error>
    + Clone
    + Send
    + Sync
    + 'static
{
    type Error;
}

impl<S: Store<StateSpace = RuntimeState>, V, Err, T> TransactionProcessor<S, V> for T
where
    V: VM,
    T: Fn(&V::Transaction, &mut [AccessHandle<S, V>]) -> Result<(), Err>
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Error = Err;
}
