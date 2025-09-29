use crate::{AccessType, ResourceId};

pub trait AccessMetadata<I: ResourceId>: Clone {
    fn resource_id(&self) -> I;

    fn access_type(&self) -> AccessType;
}
