use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

mod foreign_traits {
    use crate::{Transaction, VM};

    impl kas_l2_runtime_core::Transaction<VM> for Transaction {
        fn accessed_resources(&self) -> &[<VM as kas_l2_runtime_core::VM>::AccessMetadata] {
            &self.accessed_resources
        }
    }
}
