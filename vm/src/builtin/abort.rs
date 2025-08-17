use std::error::Error;
use std::io;

use crate::runtime_context::RuntimeContext;
use macros::builtin;
use solana_sbpf::memory_region::MemoryMapping;

#[builtin]
impl Abort {
    pub fn route(
        _ctx: &mut RuntimeContext,
        _r1: u64,
        _r2: u64,
        _r3: u64,
        _r4: u64,
        _r5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        eprintln!("[abort]");
        
        // Match panic-abort semantics by failing the VM step.
        Err(io::Error::other("abort() called").into())
    }
}
