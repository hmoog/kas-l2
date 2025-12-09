use kas_l2_vm_crypto::Address;
use kas_l2_vm_program_arg::ProgramArg;

pub enum Instruction {
    PublishProgram { program_bytes: Vec<Vec<u8>> },
    CallProgram { program_id: Address, args: Vec<ProgramArg> },
}
