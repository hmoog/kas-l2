use kas_l2_vm_pub_key::PubKey;

pub trait AuthContext {
    fn has_signer(&self, pub_key: &PubKey) -> bool;
}
