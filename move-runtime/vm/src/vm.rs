use std::sync::Arc;

use move_binary_format::errors::{VMError, VMResult};
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::move_vm::MoveVM;
use vprogs_scheduling_manager::{AccessHandle, RuntimeBatch};
use vprogs_storage_state::StateSpace;
use vprogs_storage_types::Store;

use crate::{
    ObjectAccess, ObjectId,
    execution_context::ExecutionContext,
    ownership::Ownership,
    transaction::{Transaction, TransactionEffects},
};

pub struct Vm(Arc<MoveVM>);

impl Vm {
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

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Vm {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl vprogs_scheduling_manager::VmInterface for Vm {
    type Transaction = Transaction;
    type TransactionEffects = TransactionEffects;
    type ResourceId = ObjectId;
    type Ownership = Ownership;
    type AccessMetadata = ObjectAccess;
    type Error = VMError;

    fn process_transaction<S: Store<StateSpace = StateSpace>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Vm>],
    ) -> VMResult<Self::TransactionEffects> {
        tx.instruction.execute(ExecutionContext::new(&self.0, resources))
    }

    fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, batch: &RuntimeBatch<S, Self>) {
        eprintln!(
            ">> Processed batch with {} transactions and {} state changes",
            batch.txs().len(),
            batch.state_diffs().len()
        );
    }
}
