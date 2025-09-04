use crate::errors::VMResult;
use crate::Prover;
use crate::{Executable};
use solana_sbpf::error::ProgramResult;
use solana_sbpf::vm::ContextObject;
use sp1_sdk::SP1ProofWithPublicValues;
use kas_l2_program::account_info::AccountInfo;

pub struct Program<C: ContextObject> {
    pub executable: Executable<C>,
    pub prover: Prover,
}

impl<C: ContextObject> Program<C> {
    pub fn execute(
        &self,
        ctx: &mut C,
        accounts: &[AccountInfo],
        ix_data: &[u8],
        interpreted: bool,
    ) -> (u64, ProgramResult) {
        self.executable.execute(ctx, accounts, ix_data, interpreted)
    }

    pub fn prove(
        &self,
        ctx: &mut C,
        accounts: &[AccountInfo],
        ix_data: &[u8],
    ) -> VMResult<SP1ProofWithPublicValues> {
        self.prover.prove(
            accounts,
            ix_data,
            self.execute(ctx, accounts, ix_data, false),
        )
    }

    pub fn verify(&self, proof: &SP1ProofWithPublicValues) -> VMResult<()> {
        self.prover().verify(proof)
    }

    pub fn executable(&self) -> &Executable<C> {
        &self.executable
    }

    pub fn prover(&self) -> &Prover {
        &self.prover
    }
}
