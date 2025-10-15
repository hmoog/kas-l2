use kas_l2_storage::Store;

use crate::{Batch, RuntimeState, Transaction};

pub trait BatchProcessor<S: Store<StateSpace = RuntimeState>, T: Transaction>:
    Fn(Batch<S, T>) + Clone + Send + Sync + 'static
{
}
impl<
    S: Store<StateSpace = RuntimeState>,
    T: Transaction,
    F: Fn(Batch<S, T>) + Clone + Send + Sync + 'static,
> BatchProcessor<S, T> for F
{
}
