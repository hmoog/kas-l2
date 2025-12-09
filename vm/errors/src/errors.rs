use std::io::Error;

use crate::VmError::SerializationError;

pub enum VmError {
    Generic,
    SerializationError(Error),
}

impl From<Error> for VmError {
    fn from(err: Error) -> Self {
        SerializationError(err)
    }
}
