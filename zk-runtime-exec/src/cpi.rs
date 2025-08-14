use std::io;

use solana_sbpf::{
    declare_builtin_function,
    error::EbpfError,
    memory_region::{AccessType, MemoryMapping},
    program::BuiltinProgram,
};
use solana_sbpf::error::StableResult;
use crate::exec::ExecContext;

#[inline]
fn boxed_err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(io::Error::new(io::ErrorKind::Other, msg.into()))
}

declare_builtin_function!(
    BuiltinCPI,
    fn rust(
        context_object: &mut ExecContext,
        arg_a: u64,
        arg_b: u64,
        arg_c: u64,
        arg_d: u64,
        arg_e: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let in_ptr  = arg_a;
        let in_len  = arg_b;
        let out_ptr = arg_c;
        let out_len = arg_d;
        let _unused = arg_e;

        // Map input buffer (convert ProgramResult -> boxed error)
        let in_host = match memory_mapping.map(AccessType::Load, in_ptr, in_len) {
            StableResult::Ok(addr) => addr,
            StableResult::Err(e)   => return Err(boxed_err(format!("cpi: map input failed: {e:?}"))),
        };
        let in_slice = unsafe { std::slice::from_raw_parts(in_host as *const u8, in_len as usize) };

        if in_slice.len() < 8 {
            return Err(boxed_err("cpi: input too short (< 8)"));
        }

        // Parse [app_id: u64 LE][params...]
        let mut app_id_bytes = [0u8; 8];
        app_id_bytes.copy_from_slice(&in_slice[..8]);
        let app_id = u64::from_le_bytes(app_id_bytes);
        let params = &in_slice[8..];

        // Host-side CPI "invoke"
        let out_bytes = context_object
            .cpi_invoke(app_id, params)
            .map_err(|msg| boxed_err(format!("cpi_invoke: {msg}")))?;

        // Ensure caller provided enough output space
        if out_bytes.len() as u64 > out_len {
            return Err(boxed_err(format!(
                "cpi: output buffer too small (need {}, have {})",
                out_bytes.len(),
                out_len
            )));
        }

        // Write result back into guest memory
        let out_host = match memory_mapping.map(AccessType::Store, out_ptr, out_bytes.len() as u64) {
            StableResult::Ok(addr) => addr,
            StableResult::Err(e)   => return Err(boxed_err(format!("cpi: map output failed: {e:?}"))),
        };
        unsafe {
            std::ptr::copy_nonoverlapping(out_bytes.as_ptr(), out_host as *mut u8, out_bytes.len());
        }

        Ok(0)
    }
);

/// Register the CPI builtin into a loader (concrete over ExecContext).
pub fn register_cpi_builtin(
    mut loader: BuiltinProgram<ExecContext>,
) -> Result<BuiltinProgram<ExecContext>, EbpfError> {
    loader.register_function("cpi", BuiltinCPI::vm)?;
    Ok(loader)
}
