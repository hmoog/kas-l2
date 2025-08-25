// create a public module that points to the external crate
pub mod solana_program {
    pub use solana_program::*;
}

pub mod randomness {
        pub mod custom; // defines the required hook
        pub mod buffer; // global stack of prover randomness
        #[cfg(target_os = "zkvm")]
        pub mod zkvm;
        #[cfg(not(target_os = "zkvm"))]
        pub mod host;

        pub use buffer::push_prover_randomness;
}

/// entrypoint! macro
///
/// Expands to the correct entrypoint depending on the target:
/// - On Solana BPF: expands to `entrypoint!(handler)`
/// - On SP1 zkVM:   defines a `zkvm_entry` wrapper that calls `handler`
#[macro_export]
macro_rules! entrypoint {
    ($handler:ident) => {
        #[cfg(not(target_os = "zkvm"))]
        $crate::solana_program::entrypoint!($handler);

        #[cfg(target_os = "zkvm")]
        #[unsafe(no_mangle)]
        pub extern "C" fn zkvm_entry(program_id: &[u8], accounts: &[u8], input: &[u8]) -> i32 {
            use core::convert::TryFrom;

            let program_id = $crate::pubkey::Pubkey::new_from_array(
                <[u8; 32]>::try_from(program_id).expect("invalid program_id length"),
            );
            let accounts: &[$crate::account_info::AccountInfo] = &[];
            match ($handler)(&program_id, accounts, input) {
                Ok(_) => 0,
                Err(_) => 1,
            }
        }

    };
}


// Re-export Solana program types/macros
pub use solana_program::*;

// Re-export KAS helpers
pub use randomness::push_prover_randomness;
