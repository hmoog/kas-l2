use crate::{AccessType, ResourceID};

pub trait AccessMetadata<I: ResourceID>: Clone {
    fn resource_id(&self) -> I;

    fn access_type(&self) -> AccessType;
}
