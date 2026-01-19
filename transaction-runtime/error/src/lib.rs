use std::io::Error;

use vprogs_transaction_runtime_address::Address;

use crate::VmError::SerializationError;

pub enum VmError {
    Generic,
    DataNotFound(Address),
    MissingMutCapability(Address),
    SerializationError(Error),
}

impl From<Error> for VmError {
    fn from(err: Error) -> Self {
        SerializationError(err)
    }
}

pub type VmResult<T> = Result<T, VmError>;
