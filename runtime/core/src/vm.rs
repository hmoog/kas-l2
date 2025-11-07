use kas_l2_storage_manager::Store;

use crate::{AccessHandle, AccessMetadata, ResourceId, RuntimeState, Transaction};

pub trait VM: Clone + Sized + Send + Sync + 'static {
    type Transaction: Transaction<Self>;
    type ResourceId: ResourceId;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
    type Error;

    fn process_transaction<S: Store<StateSpace = RuntimeState>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<(), Self::Error>;
}
