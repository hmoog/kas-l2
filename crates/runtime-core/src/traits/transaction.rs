use crate::{AccessMetadata, ResourceId};

pub trait Transaction: Send + Sync + 'static {
    type ResourceID: ResourceId;
    type Access: AccessMetadata<Self::ResourceID>;

    fn accessed_resources(&self) -> &[Self::Access];
}
