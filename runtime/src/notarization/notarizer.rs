use kas_l2_storage_manager::Store;

use crate::{Batch, RuntimeState, Transaction};

pub trait Notarizer<S: Store<StateSpace = RuntimeState>, T: Transaction>:
    Fn(&Batch<S, T>) + Clone + Send + Sync + 'static
{
}
impl<
    S: Store<StateSpace = RuntimeState>,
    T: Transaction,
    F: Fn(&Batch<S, T>) + Clone + Send + Sync + 'static,
> Notarizer<S, T> for F
{
}
