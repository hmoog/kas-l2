use kas_l2_runtime_interface::{AccessMetadata, Owner, ResourceId, Transaction};
use kas_l2_runtime_state::StateSpace;
use kas_l2_storage_interface::Store;

use crate::{AccessHandle, RuntimeBatch};

pub trait VM: Clone + Sized + Send + Sync + 'static {
    type Transaction: Transaction<Self::ResourceId, Self::AccessMetadata>;
    type ResourceId: ResourceId;
    type Ownership: Owner;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
    type Error;

    fn process_transaction<S: Store<StateSpace = StateSpace>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<(), Self::Error>;

    fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, batch: &RuntimeBatch<S, Self>) {
        // don't do anything by default
        let _ = batch;
    }
}
