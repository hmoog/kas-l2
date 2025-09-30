use std::marker::PhantomData;

use crate::{Runtime, Storage, Transaction, TransactionProcessor};

pub struct RuntimeBuilder<T: Transaction, S: Storage<T::ResourceID>, P: TransactionProcessor<T>> {
    pub(crate) execution_workers: usize,
    pub(crate) storage: Option<S>,
    pub(crate) transaction_processor: Option<P>,
    _marker: PhantomData<T>,
}

impl<T: Transaction, S: Storage<T::ResourceID>, P: TransactionProcessor<T>> Default
    for RuntimeBuilder<T, S, P>
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
            storage: None,
            transaction_processor: None,
            _marker: PhantomData,
        }
    }
}

impl<T: Transaction, S: Storage<T::ResourceID>, P: TransactionProcessor<T>>
    RuntimeBuilder<T, S, P>
{
    /// Override the number of execution workers.
    pub fn with_execution_workers(mut self, workers: usize) -> Self {
        self.execution_workers = workers;
        self
    }

    /// Provide a storage backend for the runtime.
    pub fn with_storage(mut self, storage: S) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Provide the transaction processor callback.
    pub fn with_transaction_processor(mut self, f: P) -> Self {
        self.transaction_processor = Some(f);
        self
    }

    /// Consume the builder and produce a runtime.
    pub fn build(self) -> Runtime<T, S> {
        Runtime::new(self)
    }
}
