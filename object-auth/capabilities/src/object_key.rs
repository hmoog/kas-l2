use std::{any::Any, fmt::Debug};

use crate::{auth_context::AuthContext, errors::AuthResult, signature_type::PubkeyType};

pub trait ObjectKey<S: PubkeyType>: Debug + Sized + Any + Send + 'static {
    type Config: Any + Send + 'static;

    fn retrieve(ctx: &mut AuthContext<S>, cfg: &Self::Config) -> AuthResult<Self>;
}
