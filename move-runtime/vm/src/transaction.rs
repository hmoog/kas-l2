use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

mod foreign_traits {
    use crate::{ObjectAccess, ObjectId, Transaction};

    impl kas_l2_runtime::Transaction for Transaction {
        type ResourceId = ObjectId;
        type AccessMetadata = ObjectAccess;

        fn accessed_resources(&self) -> &[Self::AccessMetadata] {
            &self.accessed_resources
        }
    }
}
