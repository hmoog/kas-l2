use std::error::Error;
use std::io;

use crate::runtime_context::RuntimeContext;
use kas_l2_vm_macros::builtin;
use solana_sbpf::memory_region::MemoryMapping;

#[builtin]
impl SolPanic {
    /// r1 = file_ptr, r2 = file_len, r3 = line, r4 = column
    pub fn route(
        _ctx: &mut RuntimeContext,
        file_ptr: u64,
        file_len: u64,
        line: u64,
        column: u64,
        _r5: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        eprintln!("[sol_panic_]");

        let mut name = Vec::with_capacity(file_len as usize);
        for i in 0..file_len {
            let byte = (memory_mapping.load::<u64>(file_ptr + i).unwrap() & 0xff) as u8;
            name.push(byte);
        }
        let file = String::from_utf8_lossy(&name);
        eprintln!("[sol_panic_] {file}:{line}:{column}");
        Err(io::Error::other(format!("sol_panic at {file}:{line}:{column}")).into())
    }
}
