use crate::resources::{AccessMetadata, ResourceID};

pub trait Transaction: Send + Sync + 'static {
    type ResourceID: ResourceID;
    type AccessMetadata: AccessMetadata<Self::ResourceID>;

    fn accessed_resources(&self) -> &[Self::AccessMetadata];
}
