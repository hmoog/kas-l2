use kas_l2_storage::Store;

use crate::{Batch, RuntimeState, Transaction};

pub trait BatchPostProcessor<S: Store<StateSpace = RuntimeState>, T: Transaction>:
    Fn(&Batch<S, T>) + Clone + Send + Sync + 'static
{
}
impl<
    S: Store<StateSpace = RuntimeState>,
    T: Transaction,
    F: Fn(&Batch<S, T>) + Clone + Send + Sync + 'static,
> BatchPostProcessor<S, T> for F
{
}
