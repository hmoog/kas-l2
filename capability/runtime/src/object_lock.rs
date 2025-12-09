use std::any::{TypeId};

use kas_l2_capability_errors::CapResult;

use crate::{
    AuthContext, PubkeyType, capability::Capability, erased_object_lock::ErasedObjectLock,
};

pub struct SignatureLock<P: PubkeyType> {
    pub signature: Vec<u8>,
    pub pubkey: P,
}

pub enum ObjectLockType<P: PubkeyType> {
    SignatureLock(SignatureLock<P>),
}

impl <P: PubkeyType> ObjectLockType<P> {
    pub fn unlock(&self, auth_ctx: &AuthContext<P>) -> CapResult<Box<dyn ErasedObjectLock<P>>> {
        match self {
            ObjectLockType::SignatureLock(lock) => lock.to_erased(),
        }
    }
}

pub trait ObjectLock<P: PubkeyType>: 'static {
    type Capability: Capability<P>;

    fn cap_type(&self) -> TypeId {
        TypeId::of::<Self::Capability>()
    }

    fn unlock(&self, auth_ctx: &AuthContext<P>) -> CapResult<Self::Capability>;

    fn to_erased(self) -> Box<dyn ErasedObjectLock<P>>;
}
