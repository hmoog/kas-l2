use kas_l2_runtime_manager::VmInterface;
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_types::Store;
use kas_l2_transaction_runtime_auth_context::AuthContext;
use kas_l2_transaction_runtime_lock::Lock;
use kas_l2_transaction_runtime_object_id::ObjectId;
use kas_l2_transaction_runtime_pubkey::PubKey;

use crate::TransactionRuntime;

impl<'a, 'b, S, V> AuthContext for TransactionRuntime<'a, 'b, S, V>
where
    S: Store<StateSpace = StateSpace>,
    V: VmInterface<ResourceId = ObjectId, Ownership = Lock>,
{
    fn has_signer(&self, pub_key: &PubKey) -> bool {
        self.signers.contains(pub_key)
    }
}
