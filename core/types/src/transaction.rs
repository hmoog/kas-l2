use crate::{AccessMetadata, ResourceId};

pub trait Transaction<I: ResourceId, A: AccessMetadata<I>>: Send + Sync + 'static {
    fn accessed_resources(&self) -> &[A];
}
