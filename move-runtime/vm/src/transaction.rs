use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

mod foreign_traits {
    use crate::{ObjectAccess, Transaction, VM};
    use kas_l2_runtime_core::Transaction as TransactionTrait;

    impl TransactionTrait<VM> for Transaction {
        fn accessed_resources(&self) -> &[ObjectAccess] {
            &self.accessed_resources
        }
    }
}
