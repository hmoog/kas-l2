use crate::{batch::Batch, task::Task};

pub struct Scheduler;

impl Scheduler {
    pub fn schedule<E: Task>(elements: Vec<E>) -> Batch<E> {
        Batch::new(elements)
    }
}
