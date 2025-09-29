use crate::{AccessMetadata, ResourceId};

pub trait Transaction: Send + Sync + 'static {
    type ResourceID: ResourceId;
    type AccessMetadata: AccessMetadata<Self::ResourceID>;

    fn accessed_resources(&self) -> &[Self::AccessMetadata];
}
