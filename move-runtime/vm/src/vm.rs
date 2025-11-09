use std::sync::Arc;

use kas_l2_runtime_core::{AccessHandle, Batch};
use kas_l2_runtime_state_space::StateSpace;
use kas_l2_storage_store_interface::Store;
use move_binary_format::errors::{VMError, VMResult};
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::move_vm::MoveVM;

use crate::{
    ObjectAccess, ObjectId, execution_context::ExecutionContext, ownership::Ownership,
    transaction::Transaction,
};

pub struct VM(Arc<MoveVM>);

impl VM {
    pub fn new() -> Self {
        Self(Arc::new(
            MoveVM::new(move_stdlib_natives::all_natives(
                AccountAddress::ONE,
                move_stdlib_natives::GasParameters::zeros(),
                false,
            ))
            .expect("failed to initialize VM"),
        ))
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for VM {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl kas_l2_runtime_core::VM for VM {
    type Transaction = Transaction;
    type ResourceId = ObjectId;
    type Ownership = Ownership;
    type AccessMetadata = ObjectAccess;
    type Error = VMError;

    fn process_transaction<S: Store<StateSpace = StateSpace>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, VM>],
    ) -> VMResult<()> {
        tx.instruction.execute(ExecutionContext::new(&self.0, resources))
    }

    fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, batch: &Batch<S, Self>) {
        eprintln!(
            ">> Processed batch with {} transactions and {} state changes",
            batch.txs().len(),
            batch.state_diffs().len()
        );
    }
}
