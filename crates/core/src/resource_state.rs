use std::sync::{Arc, Weak};

use crate::ResourceID;

pub struct ResourceState<ID: ResourceID> {
    pub owner: ID,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,

    /// Rollback / history link (weak to avoid cycles)
    pub prev: Option<Weak<Self>>,
}

impl<ID: ResourceID> ResourceState<ID> {
    pub fn new(owner: ID, data: Vec<u8>, balance: u64, executable: bool) -> Self {
        Self {
            owner,
            data,
            balance,
            executable,
            prev: None,
        }
    }

    pub fn clone(self: &Arc<Self>) -> Self {
        Self {
            owner: self.owner.clone(),
            data: self.data.clone(),
            balance: self.balance,
            executable: self.executable,
            prev: Some(Arc::downgrade(self)),
        }
    }
}
