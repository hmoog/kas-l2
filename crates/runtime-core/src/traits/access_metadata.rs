use crate::{ResourceId, resources::access_type::AccessType};

pub trait AccessMetadata<I: ResourceId>: Sync + Send + Clone {
    fn resource_id(&self) -> I;

    fn access_type(&self) -> AccessType;
}
