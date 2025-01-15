use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{Error, SingleValueWireFormat, WireFormat};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct TransferDataRequest {
    pub sequence: u8,
    pub data: Vec<u8>,
}

impl TransferDataRequest {
    pub(crate) fn new(sequence: u8, data: Vec<u8>) -> Self {
        Self { sequence, data }
    }
}

impl WireFormat<'_, Error> for TransferDataRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let sequence = reader.read_u8()?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(Self { sequence, data }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.sequence)?;
        writer.write_all(&self.data)?;
        Ok(1 + self.data.len())
    }
}

impl SingleValueWireFormat<'_, Error> for TransferDataRequest {}
