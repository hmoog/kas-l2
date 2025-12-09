use std::any::{Any, TypeId};

use kas_l2_capability_errors::CapResult;

use crate::{AuthContext, Capability, ObjectLock, PubkeyType};
use crate::erased_capability::ErasedCapability;

pub trait ErasedObjectLock<P: PubkeyType>: Send {
    fn key_type(&self) -> TypeId;

    fn unlock_boxed(&self, auth_ctx: &AuthContext<P>) -> CapResult<ErasedCapability>;
}

impl<P: PubkeyType, L: ObjectLock<P> + Send + 'static> ErasedObjectLock<P> for L
where
    L::Capability: Capability<P>,
{
    fn key_type(&self) -> TypeId {
        <L as ObjectLock<P>>::cap_type(self)
    }

    fn unlock_boxed(&self, auth_ctx: &AuthContext<P>) -> CapResult<ErasedCapability> {
        self.unlock(auth_ctx).map(|cap| Box::new(cap) as ErasedCapability)
    }
}
