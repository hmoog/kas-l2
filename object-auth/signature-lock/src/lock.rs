use std::any::TypeId;

use kas_l2_object_auth_capabilities::{AuthContext, AuthError, AuthResult, ObjectKey, PubkeyType};

use crate::SignatureKey;

impl<PubKey: PubkeyType> ObjectKey<PubKey> for SignatureKey {
    type Config = PubKey;

    fn retrieve(auth_ctx: &mut AuthContext<PubKey>, pubkey: &PubKey) -> AuthResult<Self> {
        if !auth_ctx.has_signature(pubkey) {
            return Err(AuthError::MissingKey(TypeId::of::<Self>()));
        }

        Ok(SignatureKey)
    }
}

pub struct SignatureLock<P: PubkeyType>(P);

impl<P: PubkeyType> SignatureLock<P> {
    pub fn new(pubkey: P) -> Self {
        Self(pubkey)
    }

    pub fn unlock(&self, auth_ctx: &mut AuthContext<P>) -> AuthResult<SignatureKey> {
        if !auth_ctx.has_signature(&self.0) {
            return Err(AuthError::MissingKey(TypeId::of::<Self>()));
        }

        Ok(SignatureKey)
    }
}
