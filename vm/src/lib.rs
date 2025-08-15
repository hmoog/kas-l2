//! zk-runtime-exec: minimal sBPF exec runtime hosting a `cpi` builtin.
//!
//! - Uses `solana-sbpf` 0.12.2 BuiltinProgram + `declare_builtin_function!`.
//! - Guest calls `cpi(in_ptr, in_len, out_ptr, out_len, 0)`.
//!   Buffer layout at `in_ptr`: [app_id: u64 LE][params: bytes].
//! - Host "invokes" callee via `ExecContext::cpi_invoke` (stubbed here).
//!
//! See the module docs and `exec::execute_elf` to run a guest ELF.

mod app_registry;
mod builtin {
    pub mod cpi;
}

mod runtime_context;
mod runtime_state;
mod vm;

pub use vm::VM;

pub use app_registry::{AppRegistry, InMemoryAppRegistry};
pub use runtime_context::RuntimeContext;
pub use runtime_state::RuntimeState;
