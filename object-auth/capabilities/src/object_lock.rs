use std::any::{Any, TypeId};

use crate::{object_key::ObjectKey, signature_type::PubkeyType};

pub struct ObjectLock {
    pub key_id: TypeId,
    pub key_config: Box<dyn Any + Send>,
}

impl ObjectLock {
    pub fn new<K: ObjectKey<S>, S: PubkeyType>(cfg: K::Config) -> Self {
        Self { key_id: TypeId::of::<K>(), key_config: Box::new(cfg) }
    }
}

mod experiment {
    use std::{
        any::{Any, TypeId},
        fmt::Debug,
    };

    use crate::{AuthContext, AuthResult, PubkeyType};

    pub trait ObjectLock<P: PubkeyType> {
        type Key: Debug + Sized + Any + Send + 'static;

        fn unlock(&self, auth_ctx: AuthContext<P>) -> AuthResult<Self::Key>;

        fn key_type(&self) -> TypeId {
            TypeId::of::<Self::Key>()
        }

        fn to_erased(self) -> Box<dyn Any + Send>;
    }
}
