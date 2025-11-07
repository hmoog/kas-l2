use crate::{AccessMetadata, ResourceId, Transaction};

pub trait VM: Sized + Send + Sync + 'static {
    type Transaction: Transaction<Self>;
    type ResourceId: ResourceId;
    type AccessMetadata: AccessMetadata<Self::ResourceId>;
}
