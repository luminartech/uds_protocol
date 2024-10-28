use crate::{EcuResetType, Error};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct EcuReset {
    pub reset_type: EcuResetType,
    _private: (),
}

impl EcuReset {
    pub(crate) fn new(reset_type: EcuResetType) -> Self {
        Self {
            reset_type,
            _private: (),
        }
    }
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let reset_type = EcuResetType::from(buffer.read_u8()?);
        Ok(Self {
            reset_type,
            _private: (),
        })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        Ok(())
    }
}
