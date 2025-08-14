use solana_sbpf::{
    vm::{Config, SyscallObject},
    error::EbpfError,
};
use crate::{state::RuntimeState, cpi::log_and_exec_cpi};

pub struct SyscallCpi<'a> {
    pub runtime_state: &'a mut RuntimeState,
}

impl<'a> SyscallObject<()> for SyscallCpi<'a> {
    fn call(
        &mut self,
        _ctx: &mut (),
        callee_ptr: u64,
        entry_ptr: u64,
        args_ptr: u64,
        _arg_len: u64,
    ) -> Result<u64, EbpfError> {
        // In real implementation, load callee_id, entry string, and args from VM memory
        let callee_app_id = [0u8; 32];
        let entry_str = "main";
        let args_bytes = vec![];

        let _output = log_and_exec_cpi(
            self.runtime_state,
            callee_app_id,
            entry_str,
            args_bytes,
        );

        Ok(0)
    }
}
