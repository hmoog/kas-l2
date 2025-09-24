use std::sync::Arc;

use crate::{
    AccessMetadata, AccessType, Transaction,
    resources::resource_handle::{ReadHandle, ResourceHandle, WriteHandle},
};

pub struct ResourceState<T: Transaction> {
    pub owner: T::ResourceID,
    pub data: Vec<u8>,
    pub balance: u64,
    pub executable: bool,
}

impl<T: Transaction> ResourceState<T> {
    pub fn new(owner: T::ResourceID, data: Vec<u8>, balance: u64, executable: bool) -> Self {
        Self {
            owner,
            data,
            balance,
            executable,
        }
    }

    pub fn cow_handle(self: &Arc<Self>, access_metadata: T::AccessMetadata) -> ResourceHandle<T> {
        match access_metadata.access_type() {
            AccessType::Read => ResourceHandle::Read(ReadHandle {
                state: Arc::clone(self),
                access_metadata,
            }),
            AccessType::Write => ResourceHandle::Write(WriteHandle {
                state: Self::clone(self),
                access_metadata,
            }),
        }
    }
}

impl<T: Transaction> Clone for ResourceState<T> {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            data: self.data.clone(),
            balance: self.balance,
            executable: self.executable,
        }
    }
}
