use std::cell::OnceCell;

use solana_program::account_info::AccountInfo;
use solana_sbpf::error::ProgramResult;
use sp1_sdk::{EnvProver, SP1ProofWithPublicValues, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};

use crate::errors::{
    VMError::{SP1, SP1Verification},
    VMResult,
};

pub struct Prover {
    pub id: [u8; 32],
    pub elf_bytes: Vec<u8>,
    pub client: OnceCell<EnvProver>,
    pub keys: OnceCell<(SP1ProvingKey, SP1VerifyingKey)>,
}

impl Prover {
    pub fn new(id: [u8; 32], bytes: &[u8]) -> Self {
        Self {
            id,
            elf_bytes: bytes.to_vec(),
            client: OnceCell::new(),
            keys: OnceCell::new(),
        }
    }

    pub fn prove(
        &self,
        acts: &[AccountInfo],
        ix_data: &[u8],
        _effects_transcript: (u64, ProgramResult),
    ) -> VMResult<SP1ProofWithPublicValues> {
        self.client()
            .prove(self.proving_key(), &self.serialize_inputs(acts, ix_data))
            .run()
            .map_err(SP1)
    }

    pub fn verify(&self, proof: &SP1ProofWithPublicValues) -> VMResult<()> {
        self.client()
            .verify(proof, self.verifying_key())
            .map_err(SP1Verification)
    }

    pub fn execute(&self, accounts: &[AccountInfo], ix_data: &[u8]) {
        let result = self
            .client()
            .execute(&self.elf_bytes, &self.serialize_inputs(accounts, ix_data))
            .run();

        match result {
            Ok((_output, _report)) => {
                println!("Program executed successfully.");
            }
            Err(err) => {
                panic!("Failed to execute SP1 program: {}.", err);
            }
        }
    }

    pub fn proving_key(&self) -> &SP1ProvingKey {
        &self
            .keys
            .get_or_init(|| self.client().setup(&self.elf_bytes))
            .0
    }

    pub fn verifying_key(&self) -> &SP1VerifyingKey {
        &self
            .keys
            .get_or_init(|| self.client().setup(&self.elf_bytes))
            .1
    }

    fn client(&self) -> &EnvProver {
        self.client.get_or_init(EnvProver::new)
    }

    fn serialize_inputs(&self, accounts: &[AccountInfo], ix_data: &[u8]) -> SP1Stdin {
        let mut stdin = SP1Stdin::new();

        stdin.write(&(accounts.len() as u64));
        // for acc in accounts {
        //     stdin.write(&[acc.dup_flag, 0,0,0,0,0,0,0]);     // [u8; 8]
        //     stdin.write(&[acc.flags,    0,0,0,0]);           // [u8; 5]
        //     stdin.write(&acc.key_bytes);                     // [u8; 32]
        //     stdin.write(&acc.owner_bytes);                   // [u8; 32]
        //     stdin.write(&acc.lamports);                      // u64
        //     stdin.write(&(acc.data.len() as u64));          // u64
        //     stdin.write_slice(&acc.data);                    // EXACTLY data.len() bytes
        // }
        stdin.write(&(ix_data.len() as u64)); // u64
        stdin.write_slice(ix_data); // exactly instr.len()
        stdin.write(&self.id); // [u8; 32]

        stdin
    }
}
