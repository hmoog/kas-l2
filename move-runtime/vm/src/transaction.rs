use move_core_types::runtime_value::MoveTypeLayout;

use crate::{Instruction, object_access::ObjectAccess};

pub struct Transaction {
    pub accessed_resources: Vec<ObjectAccess>,
    pub instruction: Instruction,
}

pub type TransactionEffects = Vec<Vec<(Vec<u8>, MoveTypeLayout)>>;

mod foreign_traits {
    use vprogs_scheduling_scheduler::VmInterface;

    use crate::{ObjectAccess, ObjectId, Transaction, Vm};

    impl vprogs_scheduling_types::Transaction<ObjectId, ObjectAccess> for Transaction {
        fn accessed_resources(&self) -> &[<Vm as VmInterface>::AccessMetadata] {
            &self.accessed_resources
        }
    }
}
