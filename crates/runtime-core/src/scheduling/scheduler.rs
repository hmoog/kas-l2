use kas_l2_storage::{Storage, Store};

use crate::{
    Batch, ResourceProvider, Transaction,
    storage::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

pub struct Scheduler<S: Store<StateSpace = RuntimeState>, T: Transaction> {
    resource_provider: ResourceProvider<S, T>,
}

impl<S: Store<StateSpace = RuntimeState>, T: Transaction> Scheduler<S, T> {
    pub fn new(resource_provider: ResourceProvider<S, T>) -> Self {
        Self { resource_provider }
    }

    pub fn schedule(
        &mut self,
        io: &Storage<S, Read<S, T>, Write<S, T>>,
        tasks: Vec<T>,
    ) -> Batch<S, T> {
        Batch::new(io, tasks, &mut self.resource_provider)
    }
}
