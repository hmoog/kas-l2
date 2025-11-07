use std::marker::PhantomData;

use kas_l2_runtime_core::{
    Batch, Notarizer, Runtime, RuntimeState, Transaction, TransactionProcessor,
};
use kas_l2_storage_manager::{StorageConfig, Store};

pub struct RuntimeBuilder<
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<S, T>,
    N: Notarizer<S, T>,
> {
    pub(crate) execution_workers: usize,
    pub(crate) transaction_processor: Option<P>,
    pub(crate) notarizer: N,
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
            notarizer: move |_| {},
            storage_config: StorageConfig::default(),
            _marker: PhantomData,
        }
    }
}

impl<
    T: Transaction,
    S: Store<StateSpace = RuntimeState>,
    P: TransactionProcessor<S, T>,
    N: Notarizer<S, T>,
> RuntimeBuilder<T, S, P, N>
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
    pub fn with_notarization<NewNotarizer: Notarizer<S, T>>(
        self,
        notarizer: NewNotarizer,
    ) -> RuntimeBuilder<T, S, P, NewNotarizer> {
        RuntimeBuilder {
            execution_workers: self.execution_workers,
            transaction_processor: self.transaction_processor,
            notarizer,
            storage_config: self.storage_config,
            _marker: PhantomData,
        }
    }

    pub fn with_storage_config(mut self, config: StorageConfig<S>) -> Self {
        self.storage_config = config;
        self
    }

    pub fn build(self) -> Runtime<S, T> {
        let RuntimeBuilder {
            execution_workers,
            transaction_processor,
            notarizer,
            storage_config,
            _marker: _,
        } = self;

        let transaction_processor =
            transaction_processor.expect("Processor must be provided before calling build()");

        Runtime::from_parts(
            execution_workers,
            transaction_processor,
            notarizer,
            storage_config,
        )
    }
}
