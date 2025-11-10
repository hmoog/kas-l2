use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

mod foreign_traits {
    use crate::{ObjectAccess, ObjectId, Transaction, VM};

    impl kas_l2_runtime_interface::Transaction<ObjectId, ObjectAccess> for Transaction {
        fn accessed_resources(&self) -> &[<VM as kas_l2_runtime_manager::VM>::AccessMetadata] {
            &self.accessed_resources
        }
    }
}
