use std::{marker::PhantomData, sync::Arc};

use crate::{batch::Batch, task::Task};

pub struct Scheduler<T: Task> {
    _marker: PhantomData<T>,
}

impl<T: Task> Scheduler<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn schedule<E: Task>(&self, elements: Vec<E>) -> Arc<Batch<E>> {
        Arc::new(Batch::new(elements))
    }
}

impl<T: Task> Default for Scheduler<T> {
    fn default() -> Self {
        Self::new()
    }
}
