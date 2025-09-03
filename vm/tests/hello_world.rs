use kas_l2_vm::{InMemoryAppRegistry, Loader, RuntimeContext, RuntimeState};
use solana_sbpf::vm::Config;
use std::sync::Arc;

#[test]
fn test_vm() {
    let loader = Loader::new(Config::default());

    println!(
        "loading program from {}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../target/kas/hello_world.kas")
    );

    let program = loader
        .load_program_file(
            [0; 32],
            concat!(env!("CARGO_MANIFEST_DIR"), "/../target/kas/hello_world.kas"),
        )
        .expect("failed to load program");

    println!("executing program");

    let (used_gas, return_value) = program.execute(
        &mut RuntimeContext::new(
            Arc::new(InMemoryAppRegistry::new()),
            RuntimeState::default(),
            10000000,
        ),
        &[],
        &[],
        true,
    );

    if return_value.is_err() {
        panic!("program execution failed with error: {return_value:?} and used gas: {used_gas}" );
    }
    println!("program returned: {return_value:?} and used gas: {used_gas}");

    println!("proving program");

    let proof = program
        .prove(
            &mut RuntimeContext::new(
                Arc::new(InMemoryAppRegistry::new()),
                RuntimeState::default(),
                10000000,
            ),
            &[],
            &[],
        )
        .expect("failed to generate proof");

    println!("verifying proof");

    program.verify(&proof).expect("failed to verify proof");

    println!("Done.");
}
