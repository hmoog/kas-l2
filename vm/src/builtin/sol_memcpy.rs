use std::error::Error;

use crate::runtime_context::RuntimeContext;
use kas_l2_vm_macros::builtin;
use solana_sbpf::memory_region::MemoryMapping;

#[builtin]
impl SolMemcpy {
    /// memcpy(dst=r1, src=r2, len=r3). Returns `dst` (on-chain behavior).
    pub fn route(
        _ctx: &mut RuntimeContext,
        dst: u64,
        src: u64,
        len: u64,
        _r4: u64,
        _r5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        eprintln!("[sol_memcpy_]");

        // Naive byte-by-byte copy using the VM memory API.
        // `load` yields u64; take the low 8 bits for a byte and store as u8.
        let mut i = 0;
        while i < len {
            let byte = (memory_mapping.load::<u64>(src + i).unwrap() & 0xff) as u8;
            memory_mapping.store::<u8>(byte, dst + i).unwrap();
            i += 1;
        }
        Ok(dst)
    }
}
