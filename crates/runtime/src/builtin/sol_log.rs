use std::error::Error;

use kas_l2_runtime_macros::builtin;
use solana_sbpf::{error::StableResult, memory_region::MemoryMapping};

use crate::runtime_context::RuntimeContext;

#[builtin]
impl SolLog {
    pub fn route(
        _ctx: &mut RuntimeContext,
        in_ptr: u64,
        in_len: u64,
        _out_ptr: u64,
        _out_len: u64,
        _unused: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        eprintln!("[sol_log_]");

        // Copy exactly in_len bytes from guest memory (cap at 1024 for safety)
        let max = core::cmp::min(in_len as usize, 1024);
        let mut buf = [0u8; 1024];
        for i in 0..max {
            match memory_mapping.load::<u64>(in_ptr + i as u64) {
                StableResult::Ok(w) => buf[i] = (w & 0xff) as u8,
                StableResult::Err(e) => {
                    eprintln!("[sol_log_] load failed @ {:#x}: {:?}", in_ptr + i as u64, e);
                    // print what we have so far and return
                    eprintln!("[sol_log_] {}", String::from_utf8_lossy(&buf[..i]));
                    return Ok(0);
                }
            }
        }

        // Log exactly the provided length (don’t scan for NUL)
        eprintln!("[sol_log_] {}", String::from_utf8_lossy(&buf[..max]));
        Ok(0)
    }
}
