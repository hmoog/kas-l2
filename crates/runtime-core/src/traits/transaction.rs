use crate::{AccessMetadata, ResourceId};

pub trait Transaction: Send + Sync + 'static {
    type ResourceId: ResourceId;
    type Access: AccessMetadata<Self::ResourceId>;

    fn accessed_resources(&self) -> &[Self::Access];
}
