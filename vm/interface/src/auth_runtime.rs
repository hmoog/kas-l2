use kas_l2_vm_crypto::PubKey;

pub trait AuthContext {
    fn has_signer(&self, pub_key: &PubKey) -> bool;
}
