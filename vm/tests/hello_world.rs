use std::fs;
use solana_sbpf::vm::Config;
use std::sync::Arc;
use vm::{Loader, RuntimeContext};
use sp1_sdk;
use sp1_sdk::{ProverClient, SP1Stdin};

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

    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Setup the prover client.
    let client = ProverClient::from_env();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();

    stdin.write(&0u64);
    // for acc in accounts {
    //     stdin.write(&[acc.dup_flag, 0,0,0,0,0,0,0]);     // [u8; 8]
    //     stdin.write(&[acc.flags,    0,0,0,0]);           // [u8; 5]
    //     stdin.write(&acc.key_bytes);                     // [u8; 32]
    //     stdin.write(&acc.owner_bytes);                   // [u8; 32]
    //     stdin.write(&acc.lamports);                      // u64
    //     stdin.write(&(acc.data.len() as u64));          // u64
    //     stdin.write_slice(&acc.data);                    // EXACTLY data.len() bytes
    // }
    stdin.write(&0u64);                  // u64
    stdin.write_slice(&[]);                           // exactly instr.len()
    stdin.write(&[0; 32]);                      // [u8; 32]

    let result = client.execute(fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../example-apps/hello-world/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/prover"
    )).expect("failed to file").as_slice(), &stdin).run();

    match result {
        Ok((_output, _report)) => {
            println!("Program executed successfully.");
        }
        Err(err) => {
            panic!("Failed to execute SP1 program: {}.", err);
        }
    }

    print!("{executed_instructions}");
}
