use kas_l2_io_core::KVStore;
use kas_l2_io_manager::IoManager;

use crate::{
    Batch, ResourceProvider, Transaction,
    io::{read_cmd::Read, runtime_state::RuntimeState, write_cmd::Write},
};

pub struct Scheduler<T: Transaction> {
    resource_provider: ResourceProvider<T>,
}

impl<T: Transaction> Scheduler<T> {
    pub fn new(resource_provider: ResourceProvider<T>) -> Self {
        Self { resource_provider }
    }

    pub fn schedule<S: KVStore<Namespace = RuntimeState>>(
        &mut self,
        io: &IoManager<S, Read<T>, Write<T>>,
        tasks: Vec<T>,
    ) -> Batch<T> {
        Batch::new(io, tasks, &mut self.resource_provider)
    }
}
