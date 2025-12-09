use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_vm_program_type::ProgramType;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Program {
    // The type of the program
    program_type: ProgramType,

    // The serialized executable bytes of the program
    elf_bytes: Vec<u8>,
}

impl Program {
    pub fn new(program_type: ProgramType, elf_bytes: Vec<u8>) -> Self {
        Self { program_type, elf_bytes }
    }

    pub fn program_type(&self) -> &ProgramType {
        &self.program_type
    }

    pub fn elf_bytes(&self) -> &Vec<u8> {
        &self.elf_bytes
    }
}
