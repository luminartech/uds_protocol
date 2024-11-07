use crate::Error;
use byteorder::{ReadBytesExt, WriteBytesExt};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum EcuResetType {
    ISOSAEReserved,
    HardReset,
    KeyOffOnReset,
    SoftReset,
}

impl From<EcuResetType> for u8 {
    fn from(value: EcuResetType) -> Self {
        match value {
            EcuResetType::ISOSAEReserved => 0x00,
            EcuResetType::HardReset => 0x01,
            EcuResetType::KeyOffOnReset => 0x02,
            EcuResetType::SoftReset => 0x03,
        }
    }
}

impl TryFrom<u8> for EcuResetType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::ISOSAEReserved),
            0x01 => Ok(Self::HardReset),
            0x02 => Ok(Self::KeyOffOnReset),
            0x03 => Ok(Self::SoftReset),
            _ => Err(Error::InvalidEcuResetType(value)),
        }
    }
}

#[non_exhaustive]
pub struct EcuResetRequest {
    pub reset_type: EcuResetType,
}

impl EcuResetRequest {
    pub(crate) fn new(reset_type: EcuResetType) -> Self {
        Self { reset_type }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let reset_type = EcuResetType::try_from(buffer.read_u8()?)?;
        Ok(Self { reset_type })
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        Ok(())
    }
}
