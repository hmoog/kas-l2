use kas_l2_vm_builtin_capabilities::AccessGranted;
use kas_l2_vm_crypto::PubKey;
use kas_l2_vm_interface::AuthContext;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SignatureLock(pub PubKey);

impl SignatureLock {
    pub fn unlock<A: AuthContext>(&self, auth_runtime: &A) -> Option<AccessGranted> {
        auth_runtime.has_signer(&self.0).then_some(AccessGranted)
    }
}
