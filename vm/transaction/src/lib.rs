use kas_l2_vm_instruction::Instruction;
use kas_l2_vm_object_access::ObjectAccess;
use kas_l2_vm_object_id::ObjectId;

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

impl kas_l2_runtime_types::Transaction<ObjectId, ObjectAccess> for Transaction {
    fn accessed_resources(&self) -> &[ObjectAccess] {
        self.accessed_objects()
    }
}
