use crate::{
    Storage, Transaction, resources::resource_provider::ResourceProvider, scheduling::batch::Batch,
};

pub struct Scheduler<T: Transaction, S: Storage<T::ResourceID>> {
    resource_provider: ResourceProvider<T, S>,
}

impl<T: Transaction, S: Storage<T::ResourceID>> Scheduler<T, S> {
    pub fn new(resource_provider: ResourceProvider<T, S>) -> Self {
        Self { resource_provider }
    }

    pub fn schedule(&mut self, tasks: Vec<T>) -> Batch<T> {
        Batch::new(tasks, &mut self.resource_provider)
    }
}
