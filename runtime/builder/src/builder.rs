use kas_l2_runtime_core::{Runtime, RuntimeState, Vm};
use kas_l2_storage_manager::{StorageConfig, Store};

pub struct RuntimeBuilder<VM: Vm, S: Store<StateSpace = RuntimeState>> {
    pub(crate) execution_workers: usize,
    pub(crate) vm: Option<VM>,
    pub(crate) storage_config: StorageConfig<S>,
}

impl<VM, S> Default for RuntimeBuilder<VM, S>
where
    VM: Vm,
    S: Store<StateSpace = RuntimeState>,
{
    fn default() -> Self {
        RuntimeBuilder {
            execution_workers: num_cpus::get_physical(),
            vm: None,
            storage_config: StorageConfig::default(),
        }
    }
}

impl<VM, S> RuntimeBuilder<VM, S>
where
    VM: Vm,
    S: Store<StateSpace = RuntimeState>,
{
    /// Override the number of execution workers.
    pub fn with_execution_workers(mut self, workers: usize) -> Self {
        self.execution_workers = workers;
        self
    }

    /// Provide the fully configured VM instance.
    pub fn with_vm(mut self, vm: VM) -> Self {
        self.vm = Some(vm);
        self
    }

    pub fn with_storage_config(mut self, config: StorageConfig<S>) -> Self {
        self.storage_config = config;
        self
    }

    pub fn build(self) -> Runtime<S, VM> {
        let RuntimeBuilder { execution_workers, vm, storage_config } = self;

        let vm = vm.expect("VM must be provided before calling build()");

        Runtime::from_parts(execution_workers, vm, storage_config)
    }
}
