use std::error::Error;
use std::io;

use crate::execution_context::ExecutionContext;
use macros::builtin;
use solana_sbpf::error::StableResult;
use solana_sbpf::memory_region::{AccessType, MemoryMapping};

#[builtin]
impl CPI {
    pub fn route(
        context_object: &mut ExecutionContext,
        in_ptr: u64,
        in_len: u64,
        out_ptr: u64,
        out_len: u64,
        _unused: u64,
        memory_mapping: &mut MemoryMapping,
    ) -> Result<u64, Box<dyn Error>> {
        #[inline]
        fn boxed_err(msg: impl Into<String>) -> Box<dyn Error> {
            Box::new(io::Error::other(msg.into()))
        }

        // Map input buffer (convert ProgramResult -> boxed error)
        let in_host = match memory_mapping.map(AccessType::Load, in_ptr, in_len) {
            StableResult::Ok(addr) => addr,
            StableResult::Err(e) => return Err(boxed_err(format!("cpi: map input failed: {e:?}"))),
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
        let out_bytes = Self::invoke(context_object, app_id, params)
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
        let out_host = match memory_mapping.map(AccessType::Store, out_ptr, out_bytes.len() as u64)
        {
            StableResult::Ok(addr) => addr,
            StableResult::Err(e) => {
                return Err(boxed_err(format!("cpi: map output failed: {e:?}")));
            }
        };
        unsafe {
            std::ptr::copy_nonoverlapping(out_bytes.as_ptr(), out_host as *mut u8, out_bytes.len());
        }

        Ok(0)
    }

    pub fn invoke(
        ctx: &mut ExecutionContext,
        app_id: u64,
        params: &[u8],
    ) -> Result<Vec<u8>, String> {
        // Demo behavior: hash(state_root || app_id || params)
        let mut hasher = blake3::Hasher::new();
        hasher.update(ctx.runtime_state.state_root.as_bytes());
        hasher.update(&app_id.to_le_bytes());
        hasher.update(params);
        let out = hasher.finalize();
        // Update the state's root to simulate a write set application
        ctx.runtime_state.state_root = out;
        Ok(out.as_bytes().to_vec()) // 32 bytes
    }
}
