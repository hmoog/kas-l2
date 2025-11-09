use crate::object_id::ObjectId;

#[derive(Clone)]
pub enum ObjectAccess {
    Read(ObjectId),
    Write(ObjectId),
}

mod foreign_traits {
    use kas_l2_runtime_interface::{AccessMetadata, AccessType};

    use crate::{ObjectAccess, ObjectId};

    impl AccessMetadata<ObjectId> for ObjectAccess {
        fn id(&self) -> ObjectId {
            match self {
                ObjectAccess::Read(object_id) => object_id.clone(),
                ObjectAccess::Write(object_id) => object_id.clone(),
            }
        }

        fn access_type(&self) -> AccessType {
            match self {
                ObjectAccess::Read(_) => AccessType::Read,
                ObjectAccess::Write(_) => AccessType::Write,
            }
        }
    }
}
