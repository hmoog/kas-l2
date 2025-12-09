use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_vm_auth_context::AuthContext;
use kas_l2_vm_builtin_capabilities::AccessGranted;
use kas_l2_vm_data::Data;
use kas_l2_vm_signature_lock::SignatureLock;

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug, BorshSerialize, BorshDeserialize)]
pub enum Lock {
    #[default]
    Shared,
    SignatureLock(SignatureLock),
}

impl Lock {
    pub fn unlock<A: AuthContext>(&self, ctx: &A) -> Option<Data> {
        match self {
            Lock::Shared => Some(Into::into(AccessGranted)),
            Lock::SignatureLock(lock) => lock.unlock(ctx).map(Into::into),
        }
    }
}
