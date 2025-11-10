use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

mod foreign_traits {
    use crate::{ObjectAccess, ObjectId, Transaction, Vm};

    impl kas_l2_runtime_common_types::Transaction<ObjectId, ObjectAccess> for Transaction {
        fn accessed_resources(
            &self,
        ) -> &[<Vm as kas_l2_runtime_manager::VmInterface>::AccessMetadata] {
            &self.accessed_resources
        }
    }
}
