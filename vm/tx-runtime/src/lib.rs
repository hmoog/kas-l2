use kas_l2_runtime_manager::{AccessHandle, RuntimeBatch, VmInterface};
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;
use kas_l2_vm_error::{VmError, VmResult};
use kas_l2_vm_lock::Lock;
use kas_l2_vm_object_access::ObjectAccess;
use kas_l2_vm_object_id::ObjectId;
use kas_l2_vm_transaction::{Transaction, TransactionEffects};
use kas_l2_vm_transaction_context::TransactionContext;

#[derive(Clone)]
pub struct TxRuntime {}

impl VmInterface for TxRuntime {
    fn process_transaction<S: Store<StateSpace = StateSpace>>(
        &self,
        tx: &Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> VmResult<TransactionEffects> {
        TransactionContext::execute(tx, resources)
    }

    fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, _batch: &RuntimeBatch<S, Self>) {}

    type Transaction = Transaction;
    type TransactionEffects = TransactionEffects;
    type ResourceId = ObjectId;
    type Ownership = Lock;
    type AccessMetadata = ObjectAccess;
    type Error = VmError;
}
