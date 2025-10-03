use std::sync::Weak;

use crate::{RuntimeTx, RuntimeTxData, Transaction};

pub struct RuntimeTxRef<Tx: Transaction>(pub(crate) Weak<RuntimeTxData<Tx>>);

impl<Tx: Transaction> RuntimeTxRef<Tx> {
    pub fn upgrade(&self) -> Option<RuntimeTx<Tx>> {
        self.0.upgrade().map(RuntimeTx)
    }
}

impl<T: Transaction> PartialEq for RuntimeTxRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Transaction> Clone for RuntimeTxRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
