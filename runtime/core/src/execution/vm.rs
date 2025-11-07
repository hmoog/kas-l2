use kas_l2_storage_manager::Store;

use crate::{AccessHandle, AccessMetadata, Batch, ResourceId, RuntimeState, Transaction};

pub trait Vm: Clone + Send + Sync + 'static {
    type ResourceId: ResourceId;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
    type Transaction: Transaction<Self>;
    type ProcessError;

    fn process<S: Store<StateSpace = RuntimeState>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<(), Self::ProcessError>;

    fn notarize<S: Store<StateSpace = RuntimeState>>(&self, batch: &Batch<S, Self>);
}
