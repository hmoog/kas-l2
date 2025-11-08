use std::io::{Read, Write};

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ownership {
    #[default]
    Shared,
    Owned,
}

impl BorshSerialize for Ownership {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Ownership::Shared => 0u8.serialize(writer),
            Ownership::Owned => 1u8.serialize(writer),
        }
    }
}

impl BorshDeserialize for Ownership {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let discriminant = u8::deserialize_reader(reader)?;
        match discriminant {
            0 => Ok(Ownership::Shared),
            1 => Ok(Ownership::Owned),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid discriminant for Ownership",
            )),
        }
    }
}
