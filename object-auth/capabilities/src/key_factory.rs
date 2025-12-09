use std::any::{Any, type_name, type_name_of_val};

use crate::{
    auth_context::AuthContext,
    errors::{AuthError, AuthResult},
    object_key::ObjectKey,
    signature_type::PubkeyType,
};

pub type KeyFactory<S> = fn(&mut AuthContext<S>, &dyn Any) -> AuthResult<Box<dyn Any + Send>>;

pub fn key_factory<C, S>(ctx: &mut AuthContext<S>, cfg: &dyn Any) -> AuthResult<Box<dyn Any + Send>>
where
    C: ObjectKey<S>,
    S: PubkeyType,
{
    match cfg.downcast_ref::<C::Config>() {
        Some(cfg) => Ok(Box::new(C::retrieve(ctx, cfg)?)),
        None => Err(AuthError::TypeMismatch {
            expected: type_name::<C::Config>(),
            found: type_name_of_val(cfg),
        }),
    }
}
