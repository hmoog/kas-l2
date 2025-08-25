use solana_sbpf::vm::Config;
use std::sync::Arc;
use vm::{Loader, RuntimeContext};
#[test]
fn test_vm() {
    let loader = Loader::new(Config::default());
    let program = loader.load_elf_file(
        [0; 32],
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../example-apps/hello-world/target/sbpf-solana-solana/release/hello_world.so"
        ),
    );

    let (executed_instructions, _result) = program.execute(
        &mut RuntimeContext::new(
            Arc::new(vm::InMemoryAppRegistry::new()),
            vm::RuntimeState::default(),
            10000000,
        ),
        &mut [],
        &mut [],
        true,
    );

    print!("{executed_instructions}");
}
