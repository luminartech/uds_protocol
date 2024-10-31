use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use crate::Error;

#[non_exhaustive]
pub struct RequestDownload {
    pub data_format_identifier: u8,
    pub address_and_length_format_identifier: u8,
    pub memory_address: u32,
    pub memory_size: u32,
}

impl RequestDownload {
    pub(crate) fn new(
        data_format_identifier: u8,
        address_and_length_format_identifier: u8,
        memory_address: u32,
        memory_size: u32,
    ) -> Self {
        Self {
            data_format_identifier,
            address_and_length_format_identifier,
            memory_address,
            memory_size,
        }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let data_format_identifier = buffer.read_u8()?;
        let address_and_length_format_identifier = buffer.read_u8()?;
        let memory_address = buffer.read_u32::<BigEndian>()?;
        let memory_size = buffer.read_u32::<BigEndian>()?;
        Ok(Self {
            data_format_identifier,
            address_and_length_format_identifier,
            memory_address,
            memory_size,
        })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(self.data_format_identifier)?;
        buffer.write_u8(self.address_and_length_format_identifier)?;
        buffer.write_u32::<BigEndian>(self.memory_address)?;
        buffer.write_u32::<BigEndian>(self.memory_size)?;
        Ok(())
    }
}
