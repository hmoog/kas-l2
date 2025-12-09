use std::{
    any::{Any, type_name_of_val},
    marker::PhantomData,
};

use crate::{
    errors::{AuthError, AuthResult},
    object_key::ObjectKey,
    signature_type::PubkeyType,
};

#[derive(Debug)]
pub struct ObjectKeyErased<S: PubkeyType>(Box<dyn Any + Send>, PhantomData<S>);

impl<S: PubkeyType> ObjectKeyErased<S> {
    pub fn try_into<C: ObjectKey<S>>(self) -> AuthResult<C> {
        self.0.downcast::<C>().map(|b| *b).map_err(|b| AuthError::TypeMismatch {
            expected: std::any::type_name::<C>(),
            found: type_name_of_val(&*b),
        })
    }
}

impl<S: PubkeyType> From<Box<dyn Any + Send>> for ObjectKeyErased<S> {
    fn from(cap: Box<dyn Any + Send>) -> Self {
        ObjectKeyErased(cap, PhantomData)
    }
}
