use kas_l2_runtime_core::{Batch, Notarizer, Runtime, RuntimeState, VM};
use kas_l2_storage_manager::{StorageConfig, Store};

pub struct RuntimeBuilder<S: Store<StateSpace = RuntimeState>, V: VM, N: Notarizer<S, V>> {
    pub(crate) execution_workers: usize,
    pub(crate) vm: Option<V>,
    pub(crate) notarizer: N,
    pub(crate) storage_config: StorageConfig<S>,
}

impl<S, V> Default for RuntimeBuilder<S, V, fn(&Batch<S, V>)>
where
    S: Store<StateSpace = RuntimeState>,
    V: VM,
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
            vm: None,
            notarizer: move |_| {},
            storage_config: StorageConfig::default(),
        }
    }
}

impl<S: Store<StateSpace = RuntimeState>, V: VM, N: Notarizer<S, V>> RuntimeBuilder<S, V, N> {
    pub fn with_execution_workers(mut self, workers: usize) -> Self {
        self.execution_workers = workers;
        self
    }

    pub fn with_vm(mut self, f: V) -> Self {
        self.vm = Some(f);
        self
    }

    pub fn with_notarization<NewNotarizer: Notarizer<S, V>>(
        self,
        notarizer: NewNotarizer,
    ) -> RuntimeBuilder<S, V, NewNotarizer> {
        RuntimeBuilder {
            execution_workers: self.execution_workers,
            vm: self.vm,
            notarizer,
            storage_config: self.storage_config,
        }
    }

    pub fn with_storage_config(mut self, config: StorageConfig<S>) -> Self {
        self.storage_config = config;
        self
    }

    pub fn build(self) -> Runtime<S, V> {
        let RuntimeBuilder { execution_workers, vm, notarizer, storage_config } = self;

        let vm = vm.expect("VM must be provided before calling build()");

        Runtime::from_parts(execution_workers, vm, notarizer, storage_config)
    }
}
