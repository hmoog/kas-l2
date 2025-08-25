/// entrypoint! macro
///
/// Expands to the correct entrypoint depending on the target:
/// - On Solana BPF: expands to `entrypoint!(handler)`
/// - On SP1 zkVM:   defines a `zkvm_entry` wrapper that calls `handler`
#[macro_export]
macro_rules! entrypoint {
    ($handler:path) => {
        #[cfg(not(target_os = "zkvm"))]
        solana_program::entrypoint!($handler);

        #[cfg(target_os = "zkvm")]
        #[no_mangle]
        pub extern "C" fn zkvm_entry(program_id: &[u8], accounts: &[u8], input: &[u8]) -> i32 {
            // TODO: Properly decode accounts & program_id
            let program_id = $crate::Pubkey::new(program_id);
            let accounts: &[$crate::AccountInfo] = &[];
            match ($handler)(&program_id, accounts, input) {
                Ok(_) => 0,
                Err(_) => 1,
            }
        }
    };
}
