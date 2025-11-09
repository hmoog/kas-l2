use kas_l2_runtime_interface::ResourceId;

use crate::AccessType;

pub trait AccessMetadata<Id: ResourceId>: Sync + Send + Clone {
    fn id(&self) -> Id;

    fn access_type(&self) -> AccessType;
}
