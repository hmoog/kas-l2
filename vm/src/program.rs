use crate::errors::VMResult;
use crate::Prover;
use crate::{Account, Executable, RuntimeContext};
use solana_sbpf::error::ProgramResult;
use sp1_sdk::SP1ProofWithPublicValues;

pub struct Program {
    pub executable: Executable,
    pub prover: Prover,
}

impl Program {
    pub fn execute(
        &self,
        ctx: &mut RuntimeContext,
        accounts: &[Account],
        ix_data: &[u8],
        interpreted: bool,
    ) -> (u64, ProgramResult) {
        self.executable.execute(ctx, accounts, ix_data, interpreted)
    }

    pub fn prove(
        &self,
        ctx: &mut RuntimeContext,
        accounts: &[Account],
        ix_data: &[u8],
    ) -> VMResult<SP1ProofWithPublicValues> {
        let _result = self.execute(ctx, accounts, ix_data, false);
        self.prover().prove(accounts, ix_data)
    }

    pub fn verify(&self, proof: &SP1ProofWithPublicValues) -> VMResult<()> {
        self.prover().verify(proof)
    }

    pub fn executable(&self) -> &Executable {
        &self.executable
    }

    pub fn prover(&self) -> &Prover {
        &self.prover
    }
}
