use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Deserialize;

use crate::{Error, SingleValueWireFormat, WireFormat};

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct WriteDataByIdentifierRequest {
    pub did: u16,
    pub data: Vec<u8>,
}

impl WriteDataByIdentifierRequest {
    pub(crate) fn new(did: u16, data: Vec<u8>) -> Self {
        Self { did, data }
    }
}

impl WireFormat<'_, Error> for WriteDataByIdentifierRequest {
    fn option_from_reader<T: std::io::Read>(buffer: &mut T) -> Result<Option<Self>, Error> {
        let did = buffer.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Some(Self { did, data }))
    }
    fn to_writer<T: std::io::Write>(&self, buffer: &mut T) -> Result<usize, Error> {
        buffer.write_u16::<BigEndian>(self.did)?;
        buffer.write_all(&self.data)?;
        Ok(2 + self.data.len())
    }
}

impl SingleValueWireFormat<'_, Error> for WriteDataByIdentifierRequest {}
