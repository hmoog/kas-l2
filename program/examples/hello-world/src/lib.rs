#![no_main]
use program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

program::entrypoint!(main);

pub fn main(_app_id: &Pubkey, _accounts: &[AccountInfo], _ix_data: &[u8]) -> ProgramResult {
    msg!("Hello, world!");

    Ok(())
}
