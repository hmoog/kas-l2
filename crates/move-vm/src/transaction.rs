use crate::{Instruction, object_access::ObjectAccess, object_id::ObjectId};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

impl kas_l2_runtime_core::Transaction for Transaction {
    type ResourceId = ObjectId;
    type AccessMetadata = ObjectAccess;

    fn accessed_resources(&self) -> &[Self::AccessMetadata] {
        &self.accessed_resources
    }
}
