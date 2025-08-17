use std::error::Error;

use crate::runtime_context::RuntimeContext;
use macros::builtin;
use solana_sbpf::memory_region::MemoryMapping;

#[builtin]
impl SolLog64 {
    /// r1..r5 are the five u64s to log
    pub fn route(
        _ctx: &mut RuntimeContext,
        r1: u64,
        r2: u64,
        r3: u64,
        r4: u64,
        r5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        eprintln!("[sol_log_64_]");
        
        eprintln!("[sol_log_64_] {r1}, {r2}, {r3}, {r4}, {r5}");
        Ok(0)
    }
}
