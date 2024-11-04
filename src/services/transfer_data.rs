use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use crate::Error;

#[non_exhaustive]
pub struct TransferDataRequest {
    pub sequence: u8,
    pub data: Vec<u8>,
}

impl TransferDataRequest {
    pub(crate) fn new(sequence: u8, data: Vec<u8>) -> Self {
        Self { sequence, data }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let sequence = buffer.read_u8()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Self { sequence, data })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(self.sequence)?;
        buffer.write_all(&self.data)?;
        Ok(())
    }
}
