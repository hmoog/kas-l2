//! zk-runtime-exec: minimal sBPF exec runtime hosting a `cpi` builtin.
//!
//! - Uses `solana-sbpf` 0.12.2 BuiltinProgram + `declare_builtin_function!`.
//! - Guest calls `cpi(in_ptr, in_len, out_ptr, out_len, 0)`.
//!   Buffer layout at `in_ptr`: [app_id: u64 LE][params: bytes].
//! - Host "invokes" callee via `ExecContext::cpi_invoke` (stubbed here).
//!
//! See the module docs and `exec::execute_elf` to run a guest ELF.

pub mod exec;
pub mod cpi;
pub mod registry;
pub mod runtime;

pub use exec::{execute_elf, ExecContext, Loader};
pub use registry::{AppRegistry, InMemoryAppRegistry};
pub use runtime::RuntimeState;