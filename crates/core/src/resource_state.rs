use std::sync::{Arc, Weak};

use crate::{
    AccessMetadata, AccessType, ResourceHandle, Transaction,
    resource_handle::{ReadHandle, WriteHandle},
};

pub struct ResourceState<T: Transaction> {
    pub owner: T::ResourceID,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,

    /// Rollback / history link (weak to avoid cycles)
    pub prev: Option<Weak<Self>>,
}

impl<T: Transaction> ResourceState<T> {
    pub fn new(owner: T::ResourceID, data: Vec<u8>, balance: u64, executable: bool) -> Self {
        Self {
            owner,
            data,
            balance,
            executable,
            prev: None,
        }
    }

    pub fn handle(self: &Arc<Self>, access_metadata: T::AccessMetadata) -> ResourceHandle<T> {
        match access_metadata.access_type() {
            AccessType::Read => ResourceHandle::Read(ReadHandle {
                state: self.clone(),
                access_metadata,
            }),
            AccessType::Write => ResourceHandle::Write(WriteHandle {
                state: Self {
                    owner: self.owner.clone(),
                    data: self.data.clone(),
                    balance: self.balance,
                    executable: self.executable,
                    prev: Some(Arc::downgrade(self)),
                },
                access_metadata,
            }),
        }
    }
}
