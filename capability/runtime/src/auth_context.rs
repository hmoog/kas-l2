use std::{any::TypeId, collections::HashMap, marker::PhantomData};
use std::any::Any;
use kas_l2_capability_errors::CapResult;
use crate::{ObjectLock, PubkeyType, erased_object_lock::ErasedObjectLock};

pub struct AuthContext<P: PubkeyType> {
    object_locks: HashMap<TypeId, Box<dyn ErasedObjectLock<P>>>,
}

impl<P: PubkeyType> AuthContext<P> {
    pub fn register_lock<L: ObjectLock<P>>(&mut self, lock: L) {
        self.object_locks.insert(TypeId::of::<L>(), lock.to_erased());
    }

    pub fn unlock_all(
        &self,
    ) -> HashMap<TypeId, CapResult<Box<dyn Any + Send>>> {
        for (id, lock) in self.object_locks {
            let (id, capability) = (*id, lock.unlock_boxed(self)?);
        }
    }
}
