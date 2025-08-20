use solana_sbpf::aligned_memory::AlignedMemory;
use solana_sbpf::ebpf;
use solana_sbpf::elf::Executable;
use solana_sbpf::error::ProgramResult;
use solana_sbpf::memory_region::{MemoryMapping, MemoryRegion};
use solana_sbpf::program::SBPFVersion;
use solana_sbpf::vm::EbpfVm;
use crate::{RuntimeContext};
use crate::account::Account;

pub struct Program {
    pub id: [u8; 32],
    pub executable: Executable<RuntimeContext>,
}

fn put_u64(mem: &mut [u8], off: usize, v: u64) {
    mem[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

fn build_input_zero_accounts(input_mem: &mut [u8], ix_data: &[u8]) -> usize {
    let need = 8 + 8 + ix_data.len() + 32;
    assert!(input_mem.len() >= need);
    let mut off = 0;
    put_u64(input_mem, off, 0); off += 8;                       // ka_num = 0
    put_u64(input_mem, off, ix_data.len() as u64); off += 8;    // data_len
    input_mem[off..off + ix_data.len()].copy_from_slice(ix_data);
    off += ix_data.len();
    input_mem[off..off + 32].fill(0);                           // program_id
    off + 32
}

impl Program {
    pub fn execute(&self, ctx: &mut RuntimeContext, _accounts: &[Account], ix_data: &[u8], interpreted: bool) -> (u64, ProgramResult) {
        const HEAP_SIZE: usize = 32 * 1024; // 32 KiB heap size
        ctx.heap_cursor = ebpf::MM_HEAP_START;
        ctx.heap_end = ebpf::MM_HEAP_START + HEAP_SIZE as u64;

        let mut input_mem = vec![0u8; 0x1000];
        let _ = build_input_zero_accounts(&mut input_mem, ix_data);
        let input_mem = input_mem.as_mut_slice();

        let config = self.executable.get_config();
        let version: SBPFVersion = self.executable.get_sbpf_version();
        let mut stack = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(config.stack_size());
        let mut heap = AlignedMemory::<{ ebpf::HOST_ALIGN }>::zero_filled(HEAP_SIZE);

        let mut vm = EbpfVm::new(
            self.executable.get_loader().clone(),
            version,
            ctx,
            MemoryMapping::new(
                vec![
                    self.executable.get_ro_region(),
                    MemoryRegion::new_writable(stack.as_slice_mut(), ebpf::MM_STACK_START),
                    MemoryRegion::new_writable(heap.as_slice_mut(), ebpf::MM_HEAP_START),
                    MemoryRegion::new_writable(input_mem, ebpf::MM_INPUT_START),
                ],
                config,
                version,
            )
                .expect("mapping"),
            stack.len(),
        );

        vm.registers[1] = ebpf::MM_INPUT_START;   // r1 = input ptr (needed on 0.12.x)
        vm.registers[2] = input_mem.len() as u64; // r2 = input len (depends on your ABI)

        vm.execute_program(&self.executable, interpreted)
    }
}