use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{Error, WireFormat};

#[non_exhaustive]
pub struct RequestDownloadRequest {
    pub data_format_identifier: u8,
    pub address_and_length_format_identifier: u8,
    pub memory_address: u32,
    pub memory_size: u32,
}

impl RequestDownloadRequest {
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
}
impl WireFormat<Error> for RequestDownloadRequest {
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let data_format_identifier = reader.read_u8()?;
        let address_and_length_format_identifier = reader.read_u8()?;
        let memory_address = reader.read_u32::<BigEndian>()?;
        let memory_size = reader.read_u32::<BigEndian>()?;
        Ok(Some(Self {
            data_format_identifier,
            address_and_length_format_identifier,
            memory_address,
            memory_size,
        }))
    }
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.data_format_identifier)?;
        writer.write_u8(self.address_and_length_format_identifier)?;
        writer.write_u32::<BigEndian>(self.memory_address)?;
        writer.write_u32::<BigEndian>(self.memory_size)?;
        Ok(10)
    }
}
