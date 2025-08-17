use std::fs;
use std::sync::Arc;
use solana_sbpf::vm::Config;
use vm::VM;
fn put_u64(mem: &mut [u8], off: usize, v: u64) {
    mem[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

/// Serialized layout expected by the loader (matches `sol_deserialize`):
/// u64 ka_num
/// u64 data_len
/// [data bytes]
/// [32] program_id
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

#[test]
fn test_vm() {
    let config = Config::default();
    // config.enable_instruction_tracing = true;

    let vm = VM::new(config);
    let app_registry = vm::InMemoryAppRegistry::new();
    let runtime_state = vm::RuntimeState::default();
    let mut ctx = vm::RuntimeContext::new(Arc::new(app_registry), runtime_state, 1000000);

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../example-apps/hello-world/target/sbpf-solana-solana/release/hello_world.so");
    let program: Vec<u8> = fs::read(path).expect("failed to read app file");

    let mut input_mem = vec![0u8; 0x1000];
    let _ = build_input_zero_accounts(&mut input_mem, &[]);

    let (executed_instructions, result) = vm.execute_elf(program.as_slice(), &mut ctx, &mut input_mem, true);

    assert!(result.is_ok(), "Program execution failed: {:?}", result);

    print!("{executed_instructions}");
}