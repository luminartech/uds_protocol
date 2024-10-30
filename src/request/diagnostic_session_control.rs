use crate::{Error, SessionType};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[non_exhaustive]
pub struct DiagnosticSessionControl {
    pub session_type: SessionType,
}

impl DiagnosticSessionControl {
    pub(crate) fn new(session_type: SessionType) -> Self {
        Self { session_type }
    }
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let session_type = SessionType::from(buffer.read_u8()?);
        Ok(Self { session_type })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.session_type))?;
        Ok(())
    }
}
