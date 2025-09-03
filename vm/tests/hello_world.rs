use kas_l2_vm::{InMemoryAppRegistry, Loader, RuntimeContext, RuntimeState};
use solana_sbpf::vm::Config;
use std::sync::Arc;

#[test]
fn test_vm() {
    let loader = Loader::new(Config::default());

    println!("=====================");
    println!(" STEP 1: Load Program");
    println!("=====================\n");

    println!(
        "-> Loading program from:\n   {}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../target/kas/hello_world.kas")
    );

    let program = loader
        .load_program_file(
            [0; 32],
            concat!(env!("CARGO_MANIFEST_DIR"), "/../target/kas/hello_world.kas"),
        )
        .expect("\n❌ Failed to load program");

    println!("\n✔ Program successfully loaded.\n");

    println!("========================");
    println!(" STEP 2: Execute Program");
    println!("========================\n");

    let (used_gas, return_value) = program.execute(
        &mut RuntimeContext::new(
            Arc::new(InMemoryAppRegistry::new()),
            RuntimeState::default(),
            10_000_000,
        ),
        &[],
        &[],
        true,
    );

    if return_value.is_err() {
        panic!(
            "\n❌ Program execution failed.\n   Error: {return_value:?}\n   Gas used: {used_gas}"
        );
    }
    println!("\n✔ Program executed successfully.");
    println!("   → Return value: {return_value:?}");
    println!("   → Gas used: {used_gas}\n");

    println!("======================");
    println!(" STEP 3: Prove Program");
    println!("======================\n");

    let proof = program
        .prove(
            &mut RuntimeContext::new(
                Arc::new(InMemoryAppRegistry::new()),
                RuntimeState::default(),
                10_000_000,
            ),
            &[],
            &[],
        )
        .expect("❌ Failed to generate proof");

    println!("\n✔ Proof successfully generated.\n");

    println!("=======================");
    println!(" STEP 4: Verify Program");
    println!("=======================\n");

    program.verify(&proof).expect("❌ Failed to verify proof");

    println!("✔ Proof successfully verified.\n");

    println!("==================");
    println!(" ✅ All steps done!");
    println!("==================");
}
