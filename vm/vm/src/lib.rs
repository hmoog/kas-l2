use kas_l2_runtime_manager::{AccessHandle, RuntimeBatch, VmInterface};
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;
use kas_l2_transaction_runtime::TransactionRuntime;
use kas_l2_transaction_runtime_error::{VmError, VmResult};
use kas_l2_transaction_runtime_lock::Lock;
use kas_l2_transaction_runtime_object_access::ObjectAccess;
use kas_l2_transaction_runtime_object_id::ObjectId;
use kas_l2_transaction_runtime_transaction::Transaction;
use kas_l2_transaction_runtime_transaction_effects::TransactionEffects;

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
    type Ownership = Lock;
    type AccessMetadata = ObjectAccess;
    type Error = VmError;
}
