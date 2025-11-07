use kas_l2_storage_manager::Store;

use crate::{Batch, RuntimeState, vm::VM};

pub trait Notarizer<S: Store<StateSpace = RuntimeState>, V: VM>:
    Fn(&Batch<S, V>) + Clone + Send + Sync + 'static
{
}
impl<
    S: Store<StateSpace = RuntimeState>,
    V: VM,
    F: Fn(&Batch<S, V>) + Clone + Send + Sync + 'static,
> Notarizer<S, V> for F
{
}
