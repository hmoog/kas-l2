use kas_l2_runtime_core::{Runtime, VM};
use kas_l2_runtime_state_space::StateSpace;
use kas_l2_runtime_storage_manager::StorageConfig;
use kas_l2_storage_interface::Store;

pub struct RuntimeBuilder<S: Store<StateSpace = StateSpace>, V: VM> {
    pub(crate) execution_workers: usize,
    pub(crate) vm: Option<V>,
    pub(crate) storage_config: StorageConfig<S>,
}

impl<S, V> Default for RuntimeBuilder<S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VM,
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
            vm: None,
            storage_config: StorageConfig::default(),
        }
    }
}

impl<S: Store<StateSpace = StateSpace>, V: VM> RuntimeBuilder<S, V> {
    pub fn with_execution_workers(mut self, workers: usize) -> Self {
        self.execution_workers = workers;
        self
    }

    pub fn with_vm(mut self, f: V) -> Self {
        self.vm = Some(f);
        self
    }

    pub fn with_storage_config(mut self, config: StorageConfig<S>) -> Self {
        self.storage_config = config;
        self
    }

    pub fn build(self) -> Runtime<S, V> {
        let RuntimeBuilder { execution_workers, vm, storage_config } = self;

        let vm = vm.expect("VM must be provided before calling build()");

        Runtime::from_parts(execution_workers, vm, storage_config)
    }
}
