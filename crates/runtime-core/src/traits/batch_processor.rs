use crate::{Transaction, scheduling::batch::Batch};

pub trait BatchProcessor<T: Transaction>: Fn(Batch<T>) + Clone + Send + Sync + 'static {}
impl<T: Transaction, F: Fn(Batch<T>) + Clone + Send + Sync + 'static> BatchProcessor<T> for F {}
