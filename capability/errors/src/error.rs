use std::any::TypeId;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CapError {
    #[error("Missing key of type {:?}", .0)]
    MissingKey(TypeId),

    #[error("Error retrieving key: {0}")]
    RetrievalError(&'static str),

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: &'static str, found: &'static str },
}
