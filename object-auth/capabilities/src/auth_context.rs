use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::{
    errors::{AuthError::MissingKey, AuthResult},
    key_factory::{KeyFactory, key_factory},
    object_key::ObjectKey,
    object_key_erased::ObjectKeyErased,
    object_lock::ObjectLock,
    signature_type::PubkeyType,
};

pub struct AuthContext<S: PubkeyType> {
    signed_pubkeys: HashSet<S>,
    key_factories: HashMap<TypeId, KeyFactory<S>>,
}

impl<S: PubkeyType> AuthContext<S> {
    pub fn new(pubkeys: &[S]) -> Self {
        Self {
            signed_pubkeys: HashSet::from_iter(pubkeys.iter().map(S::clone)),
            key_factories: HashMap::new(),
        }
    }

    pub fn register_key_type<C: ObjectKey<S>>(&mut self) {
        self.key_factories.insert(TypeId::of::<C>(), key_factory::<C, S>);
    }

    pub fn retrieve_key(&mut self, lock: &ObjectLock) -> AuthResult<ObjectKeyErased<S>> {
        let factory = self.key_factories.remove(&lock.key_id).ok_or(MissingKey(lock.key_id))?;
        let erased_key = factory(self, lock.key_config.as_ref());
        self.key_factories.insert(lock.key_id, factory);
        Ok(erased_key?.into())
    }

    pub fn new_lock<C: ObjectKey<S>>(&self, context: C::Config) -> ObjectLock {
        ObjectLock::new::<C, S>(context)
    }

    pub fn has_signature(&self, sig: &S) -> bool {
        self.signed_pubkeys.contains(sig)
    }
}
