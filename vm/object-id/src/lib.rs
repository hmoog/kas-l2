use std::io::{Read, Write};

use borsh::{BorshDeserialize, BorshSerialize};
use kas_l2_vm_address::Address;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Default)]
pub enum ObjectId {
    #[default]
    Empty,
    Program(Address),
    Data(Address),
}

impl BorshSerialize for ObjectId {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            ObjectId::Empty => 0u8.serialize(writer),
            ObjectId::Program(address) => {
                1u8.serialize(writer)?;
                address.serialize(writer)
            }
            ObjectId::Data(address) => {
                2u8.serialize(writer)?;
                address.serialize(writer)
            }
        }
    }
}

impl BorshDeserialize for ObjectId {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        match u8::deserialize_reader(reader)? {
            0 => Ok(ObjectId::Empty),
            1 => Ok(ObjectId::Program(Address::deserialize_reader(reader)?)),
            2 => Ok(ObjectId::Data(Address::deserialize_reader(reader)?)),
            discriminant => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid ObjectId discriminant: {}", discriminant),
            )),
        }
    }
}
