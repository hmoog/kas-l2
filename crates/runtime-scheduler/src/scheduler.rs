use std::sync::Arc;
use crate::{batch::Batch, task::Task};

pub struct Scheduler;

impl Scheduler {
    pub fn schedule<E: Task>(elements: Vec<E>) -> Arc<Batch<E>> {
        Arc::new(Batch::new(elements))
    }
}
