use crate::{ResourceId, resources::access_type::AccessType};

pub trait AccessMetadata<Id: ResourceId>: Sync + Send + Clone {
    fn id(&self) -> Id;

    fn access_type(&self) -> AccessType;
}
