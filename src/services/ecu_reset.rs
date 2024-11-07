use crate::Error;
use byteorder::{ReadBytesExt, WriteBytesExt};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum ResetType {
    ISOSAEReserved,
    HardReset,
    KeyOffOnReset,
    SoftReset,
}

impl From<ResetType> for u8 {
    fn from(value: ResetType) -> Self {
        match value {
            ResetType::ISOSAEReserved => 0x00,
            ResetType::HardReset => 0x01,
            ResetType::KeyOffOnReset => 0x02,
            ResetType::SoftReset => 0x03,
        }
    }
}

impl TryFrom<u8> for ResetType {
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
    pub reset_type: ResetType,
}

impl EcuResetRequest {
    pub(crate) fn new(reset_type: ResetType) -> Self {
        Self { reset_type }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let reset_type = ResetType::try_from(buffer.read_u8()?)?;
        Ok(Self { reset_type })
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn reset_type_from_all_u8_values() {
        for i in 0..=u8::MAX {
            let reset_type: Result<ResetType, Error> = ResetType::try_from(i);
            match i {
                0x00 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::ISOSAEReserved),
                )),
                0x01 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::HardReset),
                )),
                0x02 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::KeyOffOnReset),
                )),
                0x03 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::SoftReset),
                )),
                _ => assert!(matches!(
                    reset_type,
                    Err::<ResetType, Error>(Error::InvalidEcuResetType(_)),
                )),
            }
        }
    }

    #[test]
    fn reset_type_to_all_u8_values() {
        assert_eq!(u8::from(ResetType::ISOSAEReserved), 0x00);
        assert_eq!(u8::from(ResetType::HardReset), 0x01);
        assert_eq!(u8::from(ResetType::KeyOffOnReset), 0x02);
        assert_eq!(u8::from(ResetType::SoftReset), 0x03);
    }
}
