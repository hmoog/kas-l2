// create a public module that points to the external crate
pub mod solana_program {
    pub use solana_program::*;
}

// Re-export Solana program types/macros
pub use solana_program::*;

#[cfg(target_os = "zkvm")]
pub mod sp1_zkvm {
    pub use sp1_zkvm::*;
}

/// entrypoint! macro
///
/// Expands to the correct entrypoint depending on the target:
/// - On Solana BPF: expands to `entrypoint!(handler)`
/// - On SP1 zkVM:   defines a `zkvm_entry` wrapper that calls `handler`
#[macro_export]
macro_rules! entrypoint {
    ($handler:ident) => {
        // === Solana BPF target ===
        #[cfg(not(target_os = "zkvm"))]
        $crate::solana_program::entrypoint!($handler);

        // === zkVM target ===
        #[cfg(target_os = "zkvm")]
        mod zkvm_glue {
            use super::*;
            use std::cell::RefCell;
            use std::rc::Rc;

            $crate::sp1_zkvm::entrypoint!(zkvm_glue);

            pub fn zkvm_glue() {
                // ---- number of accounts ----
                let acct_count = $crate::sp1_zkvm::io::read::<u64>() as usize;

                // First pass: stable backing memory
                let mut account_keys:   Vec<$crate::pubkey::Pubkey> = Vec::with_capacity(acct_count);
                let mut account_owners: Vec<$crate::pubkey::Pubkey> = Vec::with_capacity(acct_count);

                let mut is_signer_flags:   Vec<bool> = Vec::with_capacity(acct_count);
                let mut is_writable_flags: Vec<bool> = Vec::with_capacity(acct_count);
                let mut is_exec_flags:     Vec<bool> = Vec::with_capacity(acct_count);

                let mut lamports_boxes: Vec<Box<u64>>  = Vec::with_capacity(acct_count);
                let mut data_boxes:     Vec<Box<[u8]>> = Vec::with_capacity(acct_count);

                for _ in 0..acct_count {
                    // duplicate flag + padding (8 bytes total)
                    let _dup_and_pad: [u8; 8] = $crate::sp1_zkvm::io::read();

                    // flags + 4 bytes padding (5 total); only the first byte matters
                    let flags_with_pad: [u8; 5] = $crate::sp1_zkvm::io::read();
                    let flags = flags_with_pad[0];
                    is_signer_flags.push((flags & 0b100) != 0);
                    is_writable_flags.push((flags & 0b010) != 0);
                    is_exec_flags.push((flags & 0b001) != 0);

                    // pubkey
                    let key_bytes: [u8; 32] = $crate::sp1_zkvm::io::read();
                    account_keys.push($crate::pubkey::Pubkey::new_from_array(key_bytes));

                    // owner
                    let owner_bytes: [u8; 32] = $crate::sp1_zkvm::io::read();
                    account_owners.push($crate::pubkey::Pubkey::new_from_array(owner_bytes));

                    // lamports
                    let lamports_val: u64 = $crate::sp1_zkvm::io::read();
                    lamports_boxes.push(Box::new(lamports_val));

                    // data_len then data bytes as a single chunk
                    let data_len: usize = $crate::sp1_zkvm::io::read::<u64>() as usize;
                    let data_vec: Vec<u8> = $crate::sp1_zkvm::io::read_vec();
                    assert_eq!(data_vec.len(), data_len, "mismatched account data length");
                    data_boxes.push(data_vec.into_boxed_slice());
                }

                // ---- instruction data ----
                let instr_len: usize = $crate::sp1_zkvm::io::read::<u64>() as usize;
                let instr_data: Vec<u8> = {
                    let v = $crate::sp1_zkvm::io::read_vec();
                    assert_eq!(v.len(), instr_len, "mismatched instruction data length");
                    v
                };

                // ---- program_id ----
                let pid_bytes: [u8; 32] = $crate::sp1_zkvm::io::read();
                let program_id = $crate::pubkey::Pubkey::new_from_array(pid_bytes);

                // Second pass: build AccountInfo
                let mut accounts: Vec<$crate::account_info::AccountInfo> = Vec::with_capacity(acct_count);
                for (i, (lam_box, data_box)) in lamports_boxes.iter_mut().zip(data_boxes.iter_mut()).enumerate() {
                    let lamports_cell: Rc<RefCell<&mut u64>>  = Rc::new(RefCell::new(&mut **lam_box));
                    let data_cell:     Rc<RefCell<&mut [u8]>> = Rc::new(RefCell::new(&mut **data_box));
                    #[allow(deprecated)]
                    let info = $crate::account_info::AccountInfo {
                        key: &account_keys[i],
                        is_signer: is_signer_flags[i],
                        is_writable: is_writable_flags[i],
                        lamports: lamports_cell,
                        data: data_cell,
                        owner: &account_owners[i],
                        executable: is_exec_flags[i],
                        _unused: Default::default(),
                    };
                    accounts.push(info);
                }

                // call the user handler and commit exit code
                let result = $handler(&program_id, &accounts, &instr_data);
                let exit_code: u32 = if result.is_ok() { 0 } else { 1 };
                $crate::sp1_zkvm::io::commit(&exit_code);
            }
        }
    };
}
