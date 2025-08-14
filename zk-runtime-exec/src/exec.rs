use std::sync::Arc;

use solana_sbpf::{
    aligned_memory::AlignedMemory,
    ebpf,
    elf::Executable,
    memory_region::{MemoryMapping, MemoryRegion},
    program::{BuiltinProgram, SBPFVersion},
    verifier::RequisiteVerifier,
    vm::{Config, ContextObject, EbpfVm},
    error::ProgramResult,
};

use crate::cpi::{register_cpi_builtin};
use crate::registry::AppRegistry;
use crate::runtime::RuntimeState;

/// Host-side execution context passed to the VM.
/// NOTE: non-generic; stores `Arc<dyn AppRegistry>` to satisfy `C: ContextObject`.
pub struct ExecContext {
    remaining: u64,
    pub runtime: RuntimeState,
    pub registry: Arc<dyn AppRegistry>,
}

impl ExecContext {
    pub fn new(registry: Arc<dyn AppRegistry>, runtime: RuntimeState, instruction_limit: u64) -> Self {
        Self { remaining: instruction_limit, runtime, registry }
    }

    /// Called by the CPI builtin to "invoke" another app.
    pub fn cpi_invoke(&mut self, app_id: u64, params: &[u8]) -> Result<Vec<u8>, String> {
        // Demo behavior: hash(state_root || app_id || params)
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.runtime.state_root.as_bytes());
        hasher.update(&app_id.to_le_bytes());
        hasher.update(params);
        let out = hasher.finalize();
        // Update the state's root to simulate a write set application
        self.runtime.state_root = out;
        Ok(out.as_bytes().to_vec()) // 32 bytes
    }
}

impl ContextObject for ExecContext {
    fn trace(&mut self, _state: [u64; 12]) { /* optional tracing */ }
    fn consume(&mut self, amount: u64) { self.remaining = self.remaining.saturating_sub(amount); }
    fn get_remaining(&self) -> u64 { self.remaining }
}

/// Convenience wrapper around a loader + VM config.
pub struct Loader {
    pub config: Config,
    pub builtin_loader: Arc<BuiltinProgram<ExecContext>>,
}

impl Loader {
    pub fn new_with_cpi(config: Config) -> Self {
        // Build a loader and register our CPI builtin.
        let loader = BuiltinProgram::new_loader(config.clone());
        let builtin_loader = register_cpi_builtin(loader)
            .expect("register_cpi_builtin");
        Self {
            config,
            builtin_loader: Arc::new(builtin_loader),
        }
    }
}

/// Execute a guest ELF with the given context and input buffer mapped at MM_INPUT_START.
///
/// Returns (executed_instructions, ProgramResult).
pub fn execute_elf(
    loader: &Loader,
    elf_bytes: &[u8],
    ctx: &mut ExecContext,
    input_mem: &mut [u8],
    interpreted: bool,
) -> (u64, ProgramResult) {
    // Load and verify the executable.
    let exec = Executable::<ExecContext>::from_elf(elf_bytes, loader.builtin_loader.clone())
        .expect("load executable");
    exec.verify::<RequisiteVerifier>().expect("verify elf");

    // Setup memory regions (RO program, stack, heap, input).
    let version: SBPFVersion = exec.get_sbpf_version();
    let stack_size = exec.get_config().stack_size();
    let mut stack = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(stack_size);
    let stack_len = stack.len();
    let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::with_capacity(0);

    let regions = vec![
        exec.get_ro_region(),
        MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
        MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
        MemoryRegion::new_writable(input_mem, ebpf::MM_INPUT_START),
    ];

    let mapping =
        MemoryMapping::new(regions, &exec.get_config(), version).expect("mapping");

    // Build VM and execute.
    let mut vm = EbpfVm::new(
        loader.builtin_loader.clone(),
        version,
        ctx,
        mapping,
        stack_len,
    );
    vm.execute_program(&exec, interpreted)
}
