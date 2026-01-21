use crate::{AccessType, ResourceId};

pub trait AccessMetadata<Id: ResourceId>: Sync + Send + Clone {
    fn id(&self) -> Id;

    fn access_type(&self) -> AccessType;
}
