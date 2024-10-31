use crate::Error;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[non_exhaustive]
pub struct ReadDataByIdentifierRequest {
    pub did: u16,
}

impl ReadDataByIdentifierRequest {
    pub(crate) fn new(did: u16) -> Self {
        Self { did }
    }
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let did = buffer.read_u16::<BigEndian>()?;
        Ok(Self { did })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u16::<BigEndian>(self.did)?;
        Ok(())
    }
}
