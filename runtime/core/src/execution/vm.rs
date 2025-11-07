use kas_l2_storage_manager::Store;

use crate::{AccessHandle, AccessMetadata, Batch, ResourceId, RuntimeState, Transaction};

pub trait Vm: Clone + Send + Sync + 'static {
    type ResourceId: ResourceId;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
    type Transaction: Transaction<Self>;
    type ProcessError;
    type Notarizer: Notarizer<Self>;

    fn process<S: Store<StateSpace = RuntimeState>>(
        &self,
        tx: &Self::Transaction,
        resources: &mut [AccessHandle<S, Self>],
    ) -> Result<(), Self::ProcessError>;

    fn notarizer(&self) -> &Self::Notarizer;

    fn notarize<S: Store<StateSpace = RuntimeState>>(&self, batch: &Batch<S, Self>) {
        self.notarizer().notarize(batch);
    }
}

pub trait Notarizer<VM: Vm>: Send + Sync + 'static {
    fn notarize<S: Store<StateSpace = RuntimeState>>(&self, batch: &Batch<S, VM>);
}

impl<VM: Vm> Notarizer<VM> for () {
    fn notarize<S: Store<StateSpace = RuntimeState>>(&self, _batch: &Batch<S, VM>) {}
}

impl<VM: Vm, F> Notarizer<VM> for F
where
    F: Fn(u64, usize, usize) + Send + Sync + 'static,
{
    fn notarize<S: Store<StateSpace = RuntimeState>>(&self, batch: &Batch<S, VM>) {
        self(batch.index(), batch.txs().len(), batch.state_diffs().len());
    }
}
