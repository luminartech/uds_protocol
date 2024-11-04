use crate::{EcuResetType, Error};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[non_exhaustive]
pub struct EcuResetRequest {
    pub reset_type: EcuResetType,
}

impl EcuResetRequest {
    pub(crate) fn new(reset_type: EcuResetType) -> Self {
        Self { reset_type }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let reset_type = EcuResetType::from(buffer.read_u8()?);
        Ok(Self { reset_type })
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        Ok(())
    }
}
