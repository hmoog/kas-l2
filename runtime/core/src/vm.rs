use kas_l2_storage_manager::Store;

use crate::{
    AccessHandle, AccessMetadata, Batch, ResourceId, RuntimeState, Transaction,
    data::ownership::Ownership,
};

pub trait VM: Clone + Sized + Send + Sync + 'static {
    type Transaction: Transaction<Self>;
    type ResourceId: ResourceId;
    type Ownership: Ownership;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
    type Error;

    fn process_transaction<S: Store<StateSpace = RuntimeState>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<(), Self::Error>;

    fn notarize_batch<S: Store<StateSpace = RuntimeState>>(&self, batch: &Batch<S, Self>) {
        // don't do anything by default
        let _ = batch;
    }
}
