use crate::{AccessType, ResourceID};

pub trait AccessMetadata<ID: ResourceID>: Clone {
    fn resource_id(&self) -> ID;

    fn access_type(&self) -> AccessType;
}
