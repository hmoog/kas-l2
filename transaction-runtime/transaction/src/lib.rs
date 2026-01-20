use vprogs_transaction_runtime_instruction::Instruction;
use vprogs_transaction_runtime_object_access::ObjectAccess;
use vprogs_transaction_runtime_object_id::ObjectId;

pub struct Transaction {
    accessed_objects: Vec<ObjectAccess>,
    instructions: Vec<Instruction>,
}

impl Transaction {
    pub fn new(accessed_objects: Vec<ObjectAccess>, instructions: Vec<Instruction>) -> Self {
        Transaction { accessed_objects, instructions }
    }

    pub fn accessed_objects(&self) -> &[ObjectAccess] {
        &self.accessed_objects
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

impl vprogs_scheduling_types::Transaction<ObjectId, ObjectAccess> for Transaction {
    fn accessed_resources(&self) -> &[ObjectAccess] {
        self.accessed_objects()
    }
}
