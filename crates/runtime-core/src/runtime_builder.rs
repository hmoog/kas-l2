use std::marker::PhantomData;

use crate::{
    BatchProcessor, Runtime, Storage, Transaction, TransactionProcessor, scheduling::batch::Batch,
};

pub struct RuntimeBuilder<
    T: Transaction,
    S: Storage<T::ResourceID>,
    P: TransactionProcessor<T>,
    B: BatchProcessor<T>,
> {
    pub(crate) execution_workers: usize,
    pub(crate) storage: Option<S>,
    pub(crate) transaction_processor: Option<P>,
    pub(crate) batch_processor: B,
    _marker: PhantomData<T>,
}

impl<T: Transaction, S: Storage<T::ResourceID>, P: TransactionProcessor<T>> Default
    for RuntimeBuilder<T, S, P, fn(Batch<T>)>
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
            storage: None,
            transaction_processor: None,
            batch_processor: move |_| {},
            _marker: PhantomData,
        }
    }
}

impl<T: Transaction, S: Storage<T::ResourceID>, P: TransactionProcessor<T>, B: BatchProcessor<T>>
    RuntimeBuilder<T, S, P, B>
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

    /// Provide the transaction processor callback.
    pub fn with_batch_processor<BNew: BatchProcessor<T>>(
        self,
        f: BNew,
    ) -> RuntimeBuilder<T, S, P, BNew> {
        RuntimeBuilder {
            execution_workers: self.execution_workers,
            storage: self.storage,
            transaction_processor: self.transaction_processor,
            batch_processor: f,
            _marker: PhantomData,
        }
    }

    /// Consume the builder and produce a runtime.
    pub fn build(self) -> Runtime<T, S> {
        Runtime::new(self)
    }
}
