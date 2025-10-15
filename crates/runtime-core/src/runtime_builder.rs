use std::marker::PhantomData;

use kas_l2_storage::{StorageConfig, Store};

use crate::{
    Batch, BatchProcessor, Runtime, Transaction, TransactionProcessor,
    io::runtime_state::RuntimeState,
};

pub struct RuntimeBuilder<
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<T>,
    B: BatchProcessor<T>,
> {
    pub(crate) execution_workers: usize,
    pub(crate) store: Option<S>,
    pub(crate) transaction_processor: Option<P>,
    pub(crate) batch_processor: B,
    pub(crate) storage_config: StorageConfig,
    _marker: PhantomData<T>,
}

impl<T, S, P> Default for RuntimeBuilder<T, S, P, fn(Batch<T>)>
where
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<T>,
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
            store: None,
            transaction_processor: None,
            batch_processor: move |_| {},
            storage_config: StorageConfig::default(),
            _marker: PhantomData,
        }
    }
}

impl<
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<T>,
    B: BatchProcessor<T>,
> RuntimeBuilder<T, S, P, B>
{
    /// Override the number of execution workers.
    pub fn with_execution_workers(mut self, workers: usize) -> Self {
        self.execution_workers = workers;
        self
    }

    /// Provide a storage backend for the runtime.
    pub fn with_kv_store(mut self, store: S) -> Self {
        self.store = Some(store);
        self
    }

    /// Provide the transaction processor callback.
    pub fn with_transaction_processor(mut self, f: P) -> Self {
        self.transaction_processor = Some(f);
        self
    }

    /// Provide the batch processor callback.
    pub fn with_batch_processor<BNew: BatchProcessor<T>>(
        self,
        f: BNew,
    ) -> RuntimeBuilder<T, S, P, BNew> {
        RuntimeBuilder {
            execution_workers: self.execution_workers,
            store: self.store,
            transaction_processor: self.transaction_processor,
            batch_processor: f,
            storage_config: self.storage_config,
            _marker: PhantomData,
        }
    }

    /// Consume the builder and produce a runtime.
    pub fn build(self) -> Runtime<T, S> {
        Runtime::new(self)
    }
}
