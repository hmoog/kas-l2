use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_transaction_runtime_auth_context::AuthContext;
use kas_l2_transaction_runtime_builtin_capabilities::AccessGranted;
use kas_l2_transaction_runtime_pubkey::PubKey;

#[derive(Clone, Debug, Eq, Hash, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct SignatureLock(pub PubKey);

impl SignatureLock {
    pub fn unlock<A: AuthContext>(&self, auth_ctx: &A) -> Option<AccessGranted> {
        auth_ctx.has_signer(&self.0).then_some(AccessGranted)
    }
}
