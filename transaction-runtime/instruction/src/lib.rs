use vprogs_transaction_runtime_address::Address;
use vprogs_transaction_runtime_program_arg::ProgramArg;

pub enum Instruction {
    PublishProgram { program_bytes: Vec<Vec<u8>> },
    CallProgram { program_id: Address, args: Vec<ProgramArg> },
}
