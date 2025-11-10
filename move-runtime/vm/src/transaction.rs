use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

mod foreign_traits {
    use kas_l2_runtime_manager::VmInterface;

    use crate::{ObjectAccess, ObjectId, Transaction, Vm};

    impl kas_l2_runtime_types::Transaction<ObjectId, ObjectAccess> for Transaction {
        fn accessed_resources(&self) -> &[<Vm as VmInterface>::AccessMetadata] {
            &self.accessed_resources
        }
    }
}
