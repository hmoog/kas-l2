use vprogs_scheduling_scheduler::{AccessHandle, RuntimeBatch, VmInterface};
use vprogs_storage_state::StateSpace;
use vprogs_storage_types::Store;
use vprogs_transaction_runtime::TransactionRuntime;
use vprogs_transaction_runtime_error::{VmError, VmResult};
use vprogs_transaction_runtime_object_access::ObjectAccess;
use vprogs_transaction_runtime_object_id::ObjectId;
use vprogs_transaction_runtime_transaction::Transaction;
use vprogs_transaction_runtime_transaction_effects::TransactionEffects;

#[derive(Clone)]
pub struct VM;

impl VmInterface for VM {
    fn process_transaction<S: Store<StateSpace = StateSpace>>(
        &self,
        tx: &Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> VmResult<TransactionEffects> {
        TransactionRuntime::execute(tx, resources)
    }

    fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, _batch: &RuntimeBatch<S, Self>) {}

    type Transaction = Transaction;
    type TransactionEffects = TransactionEffects;
    type ResourceId = ObjectId;
    type AccessMetadata = ObjectAccess;
    type Error = VmError;
}
