use std::io::{Read, Write};

use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_vm_auth_context::AuthContext;
use kas_l2_vm_builtin_capabilities::AccessGranted;
use kas_l2_vm_data::Data;
use kas_l2_vm_signature_lock::SignatureLock;

#[derive(Default, Clone, Hash, Eq, PartialEq, Debug)]
pub enum Lock {
    #[default]
    Shared,
    SignatureLock(SignatureLock),
}

impl Lock {
    pub fn unlock<A: AuthContext>(&self, ctx: &A) -> Option<Data> {
        match self {
            Lock::Shared => Some(Into::into(AccessGranted)),
            Lock::SignatureLock(lock) => lock.unlock(ctx).map(Into::into),
        }
    }
}

impl BorshSerialize for Lock {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Lock::Shared => 0u8.serialize(writer),
            Lock::SignatureLock(_) => 1u8.serialize(writer),
        }
    }
}

impl BorshDeserialize for Lock {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        match u8::deserialize_reader(reader)? {
            0 => Ok(Lock::Shared),
            1 => Ok(Lock::SignatureLock(SignatureLock([0; 32]))), /* Placeholder PubKey;
                                                                          * actual */
            // deserialization logic needed
            discriminant => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid ObjectLock discriminant: {}", discriminant),
            )),
        }
    }
}
