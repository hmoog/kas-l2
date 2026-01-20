use vprogs_scheduling_types::{AccessMetadata, ResourceId, Transaction};
use vprogs_storage_state::StateSpace;
use vprogs_storage_types::Store;

use crate::{AccessHandle, RuntimeBatch};

pub trait VmInterface: Clone + Sized + Send + Sync + 'static {
    fn process_transaction<S: Store<StateSpace = StateSpace>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<Self::TransactionEffects, Self::Error>;

    fn notarize_batch<S: Store<StateSpace = StateSpace>>(&self, batch: &RuntimeBatch<S, Self>) {
        if !batch.was_canceled() {
            // don't do anything by default
        }
    }

    type Transaction: Transaction<Self::ResourceId, Self::AccessMetadata>;
    type TransactionEffects: Send + Sync + 'static;
    type ResourceId: ResourceId;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
    type Error;
}
