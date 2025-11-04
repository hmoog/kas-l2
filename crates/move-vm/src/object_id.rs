use std::fmt::{Debug};
use std::hash::{Hash};
use std::io::{Read, Write};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use crate::object_id::ObjectId::Data;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum ObjectId {
    Module(ModuleId),
    Data(AccountAddress),
}

impl borsh::ser::BorshSerialize for ObjectId {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            ObjectId::Module(module_id) => {
                // Write discriminant for Module variant
                0u8.serialize(writer)?;
                // Serialize the address component
                module_id.address().serialize(writer)?;
                // Serialize the name component
                module_id.name().as_str().serialize(writer)
            }
            ObjectId::Data(address) => {
                // Write discriminant for Data variant
                1u8.serialize(writer)?;
                address.serialize(writer)
            }
        }
    }
}

impl borsh::de::BorshDeserialize for ObjectId {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        // Read the discriminant byte
        let discriminant = u8::deserialize_reader(reader)?;

        match discriminant {
            0 => {
                let mut address_bytes = [0u8; AccountAddress::LENGTH];
                reader.read_exact(&mut address_bytes)?;
                let address = AccountAddress::new(address_bytes);
                let name_str = String::deserialize_reader(reader)?;
                let name = Identifier::new(name_str)
                    .map_err(|e| std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid identifier: {}", e)
                    ))?;
                Ok(ObjectId::Module(ModuleId::new(address, name)))
            }
            1 => {
                // Data variant: deserialize address
                let mut address_bytes = [0u8; AccountAddress::LENGTH];
                reader.read_exact(&mut address_bytes)?;
                let address = AccountAddress::new(address_bytes);
                Ok(ObjectId::Data(address))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid ObjectId discriminant: {}", discriminant)
            ))
        }
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Data(AccountAddress::ZERO)
    }
}