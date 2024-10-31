use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::Error;

#[non_exhaustive]
pub struct WriteDataByIdentifier {
    pub did: u16,
    pub data: Vec<u8>,
}

impl WriteDataByIdentifier {
    pub(crate) fn new(did: u16, data: Vec<u8>) -> Self {
        Self { did, data }
    }
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let did = buffer.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Self { did, data })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u16::<BigEndian>(self.did)?;
        buffer.write_all(&self.data)?;
        Ok(())
    }
}
