use kas_l2_transaction_runtime_pubkey::PubKey;

pub trait AuthContext {
    fn has_signer(&self, pub_key: &PubKey) -> bool;
}
