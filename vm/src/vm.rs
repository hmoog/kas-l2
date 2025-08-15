use crate::ExecutionContext;
use crate::cpi::CPI;
use solana_sbpf::aligned_memory::AlignedMemory;
use solana_sbpf::ebpf;
use solana_sbpf::elf::Executable;
use solana_sbpf::error::ProgramResult;
use solana_sbpf::memory_region::{MemoryMapping, MemoryRegion};
use solana_sbpf::program::{BuiltinProgram, SBPFVersion};
use solana_sbpf::verifier::RequisiteVerifier;
use solana_sbpf::vm::{Config, EbpfVm};
use std::sync::Arc;

/// Convenience wrapper around a loader + VM config.
pub struct VM {
    pub config: Config,
    pub builtin_program: Arc<BuiltinProgram<ExecutionContext>>,
}

impl VM {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            builtin_program: Arc::new({
                let mut loader = BuiltinProgram::new_loader(config);

                loader
                    .register_function("cpi", CPI::vm)
                    .expect("register cpi function");

                loader
            }),
        }
    }

    /// Execute a guest ELF with the given context and input buffer mapped at MM_INPUT_START.
    ///
    /// Returns (executed_instructions, ProgramResult).
    pub fn execute_elf(
        &self,
        elf_bytes: &[u8],
        ctx: &mut ExecutionContext,
        input_mem: &mut [u8],
        interpreted: bool,
    ) -> (u64, ProgramResult) {
        // Load and verify the executable.
        let exec =
            Executable::<ExecutionContext>::from_elf(elf_bytes, self.builtin_program.clone())
                .expect("load executable");
        exec.verify::<RequisiteVerifier>().expect("verify elf");

        // Setup memory regions (RO program, stack, heap, input).
        let version: SBPFVersion = exec.get_sbpf_version();
        let mut stack =
            AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(exec.get_config().stack_size());
        let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::with_capacity(0);

        let regions = vec![
            exec.get_ro_region(),
            MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
            MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
            MemoryRegion::new_writable(input_mem, ebpf::MM_INPUT_START),
        ];

        let mapping = MemoryMapping::new(regions, exec.get_config(), version).expect("mapping");

        EbpfVm::new(
            self.builtin_program.clone(),
            version,
            ctx,
            mapping,
            stack.len(),
        )
        .execute_program(&exec, interpreted)
    }
}
