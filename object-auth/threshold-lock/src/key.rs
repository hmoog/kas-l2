use kas_l2_object_auth_capabilities::{AuthContext, AuthError, AuthResult, ObjectKey, PubkeyType};

use crate::lock::ThresholdLock;

#[derive(Debug)]
pub struct ThresholdKey;

impl<P: PubkeyType> ObjectKey<P> for ThresholdKey {
    type Config = ThresholdLock;

    fn retrieve(auth_ctx: &mut AuthContext<P>, cfg: &Self::Config) -> AuthResult<Self> {
        let mut valid_keys = 0;
        for child_lock in &cfg.child_locks {
            match auth_ctx.retrieve_key(child_lock) {
                Ok(_) => valid_keys = valid_keys + 1,
                Err(_) => continue,
            }
        }

        if valid_keys < cfg.threshold {
            return Err(AuthError::RetrievalError("Not enough available keys for threshold"));
        }

        Ok(ThresholdKey)
    }
}
