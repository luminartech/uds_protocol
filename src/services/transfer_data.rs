use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{Error, SingleValueWireFormat, WireFormat};

/// A request to the server to transfer data (either upload or download)
/// 
/// Step 1: The client sends a RequestDownload or RequestUpload message to the server
///     34 .. 11  .. 33   .. 60 20 00 .. 00 FF FF << -- Bytes sent by the client
///    RID .. DFI .. ALFID .. MA_B#   .. UCMS_B#
/// 
/// Step 2: The server sends a [`crate::RequestDownloadResponse`] or [`crate::RequestUploadResponse`] message to the client
///     The client shall send a TransferDataRequest message 
///     to the server with a max number of bytes equal to MNROB_B# from the RequestDownloadResponse message
///    74  .. 20   .. 00 81
///   RSID .. LFID .. MNROB_B#
/// 

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct TransferDataRequest {
    /// Starts at 0x01 from the server when a RequestDownload or RequestUpload or RequestFileTransfer is received
    /// Increments by 0x01 for each TransferDataRequest message
    /// At 0xFF the counter wraps around to 0x00
    pub block_sequence_counter: u8,
    /// The data to be transferred, the server sends the amount of data (# of bytes) it can handle in the
    /// [`crate::RequestDownloadResponse`] message
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct TransferDataResponse {
    /// Starts at 0x01 from the server when a RequestDownload or RequestUpload or RequestFileTransfer is received
    /// Increments by 0x01 for each TransferDataRequest message
    /// At 0xFF the counter wraps around to 0x00
    /// 
    /// This is an ECHO of the block_sequence_counter from the [`TransferDataRequest`] message
    /// Check against the request to ensure the correct block is being acknowledged
    /// If the block_sequence_counter is not as expected or does not arrive, the client should retransmit the block
    pub block_sequence_counter: u8,

    /// Contains data required by the client to support the transfer of data.
    /// Vehicle manufacturer specific
    /// 
    /// For download (client to server), this might be a checksum for the client to verify correct transfer
    ///     This should not repeat the data sent from the client
    /// For upload (server to client), this will include the data from the server
    pub data: Vec<u8>,
}

impl TransferDataResponse {
    pub(crate) fn new(block_sequence_counter: u8, data: Vec<u8>) -> Self {
        Self { block_sequence_counter, data }
    }
}

impl WireFormat for TransferDataResponse {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_data_request() {
        let data: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
        let request = TransferDataRequest::new(0x01, data.to_vec());
        let bytes = request.data.clone();
        let expected = vec![0x01, 0x02, 0x03, 0x04];
        assert_eq!(1, request.block_sequence_counter);
        assert_eq!(bytes, expected);

    }
}
