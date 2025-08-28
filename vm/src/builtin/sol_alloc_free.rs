use std::error::Error;

use crate::runtime_context::RuntimeContext;
use kas_l2_vm_macros::builtin;
use solana_sbpf::memory_region::MemoryMapping;

#[builtin]
impl SolAllocFree {
    /// Allocate when (size > 0 && free_ptr == 0).
    /// Free when (size == 0 && free_ptr != 0) — no-op for bump allocator.
    pub fn route(
        ctx: &mut RuntimeContext,
        size: u64,     // r1
        free_ptr: u64, // r2
        _r3: u64,
        _r4: u64,
        _r5: u64,
        _memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        eprintln!("[sol_alloc_free_]");

        // free: ignore (no-op)
        if size == 0 && free_ptr != 0 {
            return Ok(0);
        }

        // alloc: align to 8 bytes
        if size > 0 && free_ptr == 0 {
            let aligned = (size + 7) & !7;
            let next = ctx.heap_cursor.saturating_add(aligned);
            if next > ctx.heap_end {
                // OOM: return 0 per on-chain behavior
                return Ok(0);
            }
            let ret = ctx.heap_cursor;
            ctx.heap_cursor = next;
            return Ok(ret);
        }

        // Any other combination: treat as no-op
        Ok(0)
    }
}
