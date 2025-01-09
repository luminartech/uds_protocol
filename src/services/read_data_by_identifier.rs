use crate::{Error, WireFormat};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[non_exhaustive]
pub struct ReadDataByIdentifierRequest {
    pub did: u16,
}

impl ReadDataByIdentifierRequest {
    pub(crate) fn new(did: u16) -> Self {
        Self { did }
    }
}
impl WireFormat<Error> for ReadDataByIdentifierRequest {
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let did = reader.read_u16::<BigEndian>()?;
        Ok(Some(Self { did }))
    }
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u16::<BigEndian>(self.did)?;
        Ok(2)
    }
}
