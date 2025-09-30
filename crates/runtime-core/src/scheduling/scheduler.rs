use crate::{
    Storage, Transaction, resources::resource_provider::ResourceProvider, scheduling::batch::Batch,
};

pub struct Scheduler<T: Transaction, K: Storage<T::ResourceID>> {
    resource_provider: ResourceProvider<T, K>,
}

impl<T: Transaction, K: Storage<T::ResourceID>> Scheduler<T, K> {
    pub fn new(resource_provider: ResourceProvider<T, K>) -> Self {
        Self { resource_provider }
    }

    pub fn schedule(&mut self, tasks: Vec<T>) -> Batch<T> {
        Batch::new(tasks, &mut self.resource_provider)
    }
}
