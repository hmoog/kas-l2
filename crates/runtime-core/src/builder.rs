use std::marker::PhantomData;

use kas_l2_storage::{StorageConfig, Store};

use crate::{
    Batch, BatchPostProcessor, Runtime, Transaction, TransactionProcessor,
    storage::runtime_state::RuntimeState,
};

pub struct RuntimeBuilder<
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<S, T>,
    B: BatchPostProcessor<S, T>,
> {
    pub(crate) execution_workers: usize,
    pub(crate) transaction_processor: Option<P>,
    pub(crate) batch_processor: B,
    pub(crate) storage_config: StorageConfig<S>,
    _marker: PhantomData<T>,
}

impl<T, S, P> Default for RuntimeBuilder<T, S, P, fn(&Batch<S, T>)>
where
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<S, T>,
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
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
    P: TransactionProcessor<S, T>,
    B: BatchPostProcessor<S, T>,
> RuntimeBuilder<T, S, P, B>
{
    /// Override the number of execution workers.
    pub fn with_execution_workers(mut self, workers: usize) -> Self {
        self.execution_workers = workers;
        self
    }

    /// Provide the transaction processor callback.
    pub fn with_transaction_processor(mut self, f: P) -> Self {
        self.transaction_processor = Some(f);
        self
    }

    /// Provide the batch processor callback.
    pub fn with_batch_processor<BNew: BatchPostProcessor<S, T>>(
        self,
        f: BNew,
    ) -> RuntimeBuilder<T, S, P, BNew> {
        RuntimeBuilder {
            execution_workers: self.execution_workers,
            transaction_processor: self.transaction_processor,
            batch_processor: f,
            storage_config: self.storage_config,
            _marker: PhantomData,
        }
    }

    pub fn with_storage_config(mut self, config: StorageConfig<S>) -> Self {
        self.storage_config = config;
        self
    }

    /// Consume the builder and produce a runtime.
    pub fn build(self) -> Runtime<S, T> {
        Runtime::new(self)
    }
}
