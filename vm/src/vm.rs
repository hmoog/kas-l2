use crate::RuntimeContext;
use crate::builtin::cpi::CPI;
use solana_sbpf::aligned_memory::AlignedMemory;
use solana_sbpf::ebpf;
use solana_sbpf::elf::Executable;
use solana_sbpf::error::ProgramResult;
use solana_sbpf::memory_region::{MemoryMapping, MemoryRegion};
use solana_sbpf::program::{BuiltinProgram, SBPFVersion};
use solana_sbpf::verifier::RequisiteVerifier;
use solana_sbpf::vm::{Config, EbpfVm};
use std::sync::Arc;
use crate::builtin::abort::Abort;
use crate::builtin::sol_alloc_free::SolAllocFree;
use crate::builtin::sol_log::SolLog;
use crate::builtin::sol_log_64::SolLog64;
use crate::builtin::sol_memcpy::SolMemcpy;
use crate::builtin::sol_panic::SolPanic;

/// Convenience wrapper around a loader + VM config.
pub struct VM {
    pub config: Config,
    pub builtin_program: Arc<BuiltinProgram<RuntimeContext>>,
}

impl VM {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            builtin_program: Arc::new({
                let mut loader = BuiltinProgram::new_loader(config);
                loader.register_function("abort", Abort::vm).unwrap();
                loader.register_function("cpi", CPI::vm).unwrap();
                loader.register_function("sol_alloc_free_", SolAllocFree::vm).unwrap();
                loader.register_function("sol_log_", SolLog::vm).unwrap();
                loader.register_function("sol_log_64_", SolLog64::vm).unwrap();
                loader.register_function("sol_memcpy_", SolMemcpy::vm).unwrap();
                loader.register_function("sol_panic_", SolPanic::vm).unwrap();

                loader
            }),
        }
    }

    pub fn execute_elf(
        &self,
        elf_bytes: &[u8],
        ctx: &mut RuntimeContext,
        input_mem: &mut [u8],
        interpreted: bool,
    ) -> (u64, ProgramResult) {
        const HEAP_SIZE: usize = 32 * 1024; // 32 KiB heap size

        // Load and verify the executable.
        let exec =
            Executable::from_elf(elf_bytes, self.builtin_program.clone()).expect("load executable");
        exec.verify::<RequisiteVerifier>().expect("verify elf");
        //exec.get_loader();

        // Setup memory regions (RO program, stack, heap, input).
        let version: SBPFVersion = exec.get_sbpf_version();
        let mut stack =
            AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(exec.get_config().stack_size());
        let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(HEAP_SIZE);

        ctx.heap_cursor = ebpf::MM_HEAP_START;
        ctx.heap_end = ebpf::MM_HEAP_START + HEAP_SIZE as u64;

        let mut vm = EbpfVm::new(
            self.builtin_program.clone(),
            version,
            ctx,
            MemoryMapping::new(
                vec![
                    exec.get_ro_region(),
                    MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
                    MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
                    MemoryRegion::new_writable(input_mem, ebpf::MM_INPUT_START),
                ],
                exec.get_config(),
                version,
            )
            .expect("mapping"),
            stack.len(),
        );

        // Pass arguments:
        vm.registers[1] = ebpf::MM_INPUT_START;             // r1 = input ptr (needed on 0.12.x)
        // optional, if your program expects it:
        vm.registers[2] = input_mem.len() as u64;           // r2 = input len (depends on your ABI)


        vm.execute_program(&exec, interpreted)
    }
}
