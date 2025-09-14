use std::{error::Error, io};

use kas_l2_runtime_macros::builtin;
use solana_sbpf::memory_region::MemoryMapping;

use crate::runtime_context::RuntimeContext;

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
