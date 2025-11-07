use std::sync::Arc;

use kas_l2_runtime_core::{Batch, RuntimeState, Vm};
use kas_l2_storage_manager::Store;
use move_binary_format::errors::VMResult;
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::move_vm::MoveVM;

use crate::{
    execution_context::ExecutionContext, object_access::ObjectAccess, object_id::ObjectId,
    transaction::Transaction,
};

#[derive(Clone)]
pub struct VM {
    vm: Arc<MoveVM>,
    notarizer: Arc<dyn Fn(u64, usize, usize) + Send + Sync>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            vm: Arc::new(
                MoveVM::new(move_stdlib_natives::all_natives(
                    AccountAddress::ONE,
                    move_stdlib_natives::GasParameters::zeros(),
                    false,
                ))
                .expect("failed to initialize VM"),
            ),
            notarizer: Arc::new(|_, _, _| {}),
        }
    }

    pub fn with_notarizer<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, usize, usize) + Send + Sync + 'static,
    {
        self.notarizer = Arc::new(callback);
        self
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl Vm for VM {
    type ResourceId = ObjectId;
    type AccessMetadata = ObjectAccess;
    type Transaction = Transaction;
    type ProcessError = move_binary_format::errors::VMError;

    fn process<S: Store<StateSpace = RuntimeState>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [kas_l2_runtime_core::AccessHandle<S, Self>],
    ) -> Result<(), Self::ProcessError> {
        tx.instruction.execute(ExecutionContext::new(self.vm.as_ref(), resources))
    }

    fn notarize<S: Store<StateSpace = RuntimeState>>(&self, batch: &Batch<S, Self>) {
        (self.notarizer)(batch.index(), batch.txs().len(), batch.state_diffs().len());
    }
}
