use solana_sbpf::error::EbpfError;
use sp1_sdk::SP1VerificationError;

pub type VMResult<T> = Result<T, VMError>;

#[derive(Debug)]
pub enum VMError {
    Sbf(EbpfError),
    IO(std::io::Error),
    SP1(anyhow::Error),
    SP1Verification(SP1VerificationError),
}

impl From<std::io::Error> for VMError {
    fn from(e: std::io::Error) -> Self {
        VMError::IO(e)
    }
}

impl From<EbpfError> for VMError {
    fn from(e: EbpfError) -> Self {
        VMError::Sbf(e)
    }
}
