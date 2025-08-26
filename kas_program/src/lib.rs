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

            $crate::sp1_zkvm::entrypoint!(main);

            pub fn main() {
                // === number of accounts ===
                let acct_count = $crate::sp1_zkvm::io::read::<u64>() as usize;

                // ---- First pass: read and allocate stable backing memory (no leaks) ----
                let mut account_keys:   Vec<$crate::pubkey::Pubkey> = Vec::with_capacity(acct_count);
                let mut account_owners: Vec<$crate::pubkey::Pubkey> = Vec::with_capacity(acct_count);

                let mut is_signer_flags:  Vec<bool> = Vec::with_capacity(acct_count);
                let mut is_writable_flags:Vec<bool> = Vec::with_capacity(acct_count);
                let mut is_exec_flags:    Vec<bool> = Vec::with_capacity(acct_count);

                // Each account’s state lives in its own Box.
                let mut lamports_boxes: Vec<Box<u64>>   = Vec::with_capacity(acct_count);
                let mut data_boxes:     Vec<Box<[u8]>>  = Vec::with_capacity(acct_count);

                for _ in 0..acct_count {
                    // duplicate flag + padding
                    let _dup_flag = $crate::sp1_zkvm::io::read::<u8>();
                    for _ in 0..7 { let _ = $crate::sp1_zkvm::io::read::<u8>(); }

                    // flags
                    let flags = $crate::sp1_zkvm::io::read::<u8>();
                    for _ in 0..4 { let _ = $crate::sp1_zkvm::io::read::<u8>(); }
                    let is_signer     = (flags & 0b100) != 0;
                    let is_writable   = (flags & 0b010) != 0;
                    let is_executable = (flags & 0b001) != 0;
                    is_signer_flags.push(is_signer);
                    is_writable_flags.push(is_writable);
                    is_exec_flags.push(is_executable);

                    // pubkey
                    let mut key_bytes = [0u8; 32];
                    for b in &mut key_bytes { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                    account_keys.push($crate::pubkey::Pubkey::new_from_array(key_bytes));

                    // owner
                    let mut owner_bytes = [0u8; 32];
                    for b in &mut owner_bytes { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                    account_owners.push($crate::pubkey::Pubkey::new_from_array(owner_bytes));

                    // lamports -> Box<u64>
                    let mut lamports_bytes = [0u8; 8];
                    for b in &mut lamports_bytes { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                    let lamports_val = u64::from_le_bytes(lamports_bytes);
                    lamports_boxes.push(Box::new(lamports_val));

                    // data -> Box<[u8]>
                    let mut data_len_bytes = [0u8; 8];
                    for b in &mut data_len_bytes { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                    let data_len = u64::from_le_bytes(data_len_bytes) as usize;

                    let mut data_vec = vec![0u8; data_len];
                    for b in &mut data_vec { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                    data_boxes.push(data_vec.into_boxed_slice());
                }

                // ---- instruction data ----
                let mut instr_len_bytes = [0u8; 8];
                for b in &mut instr_len_bytes { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                let instr_len = u64::from_le_bytes(instr_len_bytes) as usize;

                let mut instr_data = vec![0u8; instr_len];
                for b in &mut instr_data { *b = $crate::sp1_zkvm::io::read::<u8>(); }

                // ---- program_id ----
                let mut pid_bytes = [0u8; 32];
                for b in &mut pid_bytes { *b = $crate::sp1_zkvm::io::read::<u8>(); }
                let program_id = $crate::pubkey::Pubkey::new_from_array(pid_bytes);

                // ---- Second pass: build AccountInfo (no further pushes to the box vectors) ----
                let mut accounts: Vec<$crate::account_info::AccountInfo> = Vec::with_capacity(acct_count);

                // Use iter_mut to obtain disjoint &mut to each Box’s contents.
                for (i, (lam_box, data_box)) in lamports_boxes.iter_mut().zip(data_boxes.iter_mut()).enumerate() {
                    let lamports_cell: Rc<RefCell<&mut u64>>   = Rc::new(RefCell::new(&mut **lam_box));
                    let data_cell:     Rc<RefCell<&mut [u8]>>  = Rc::new(RefCell::new(&mut **data_box));

                    #[allow(deprecated)]
                    let info = $crate::account_info::AccountInfo {
                        key: &account_keys[i],
                        is_signer: is_signer_flags[i],
                        is_writable: is_writable_flags[i],
                        lamports: lamports_cell,
                        data: data_cell,
                        owner: &account_owners[i],
                        executable: is_exec_flags[i],
                        _unused: Default::default(), // ABIv1 compat
                    };
                    accounts.push(info);
                }

                // ---- call the user handler ----
                let result = $handler(&program_id, &accounts, &instr_data);

                // ---- exit code ----
                let exit_code: u32 = if result.is_ok() { 0 } else { 1 };
                $crate::sp1_zkvm::io::commit(&exit_code);
            }
        }
    };
}
