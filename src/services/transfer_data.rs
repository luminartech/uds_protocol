use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{Error, SingleValueWireFormat, WireFormat};

#[non_exhaustive]
pub struct TransferDataRequest {
    /// Starts at 0x01 from the server when a RequestDownload or RequestUpload or RequestFileTransfer is received
    /// Increments by 0x01 for each TransferDataRequest message
    /// At 0xFF the counter wraps around to 0x00
    pub block_sequence_counter: u8,
    /// The data to be transferred
    pub data: Vec<u8>,
}

impl TransferDataRequest {
    pub(crate) fn new(block_sequence_counter: u8, data: Vec<u8>) -> Self {
        Self { block_sequence_counter, data }
    }
}

impl WireFormat for TransferDataRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let block_sequence_counter = reader.read_u8()?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(Self { block_sequence_counter, data }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.block_sequence_counter)?;
        writer.write_all(&self.data)?;
        Ok(1 + self.data.len())
    }
}

impl SingleValueWireFormat for TransferDataRequest {}
