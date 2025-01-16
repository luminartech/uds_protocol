use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    DataFormatIdentifier, Error, LengthFormatIdentifier, MemoryFormatIdentifier,
    NegativeResponseCode, SingleValueWireFormat, WireFormat,
};

const REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 6] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::AuthenticationRequired,
    NegativeResponseCode::UploadDownloadNotAccepted,
];

/// A request to the server for it to download data from the client
///
/// A positive response to this request ([`RequestDownloadResponse`]) will happen
/// after the server takes all necessary actions to receive the data once the server is ready to receive
///
/// This is a variable length Request, determined by the `address_and_length_format_identifier` value
/// See ISO-14229-1:2020, Table H.1 for format information
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RequestDownloadRequest {
    /// compression method (high nibble) and encrypting method (low nibble). 0x00 is no compression or encryption
    data_format_identifier: DataFormatIdentifier,
    /// 7-4: length (# of bytes) of memory_size param, 3-0: length (# of bytes) of memory_address param
    address_and_length_format_identifier: MemoryFormatIdentifier,
    /// Starting address of the server memory. Size is determined by `address_and_length_format_identifier`
    /// Has a variable number of bytes, max of 5
    pub memory_address: u64,
    /// Size of the data to be downloaded. Number of bytes sent is determined by `address_and_length_format_identifier`
    /// Used by the server to validate the data transferred by the [`TransferData`] service
    /// Has a variable number of bytes, max of 4
    pub memory_size: u32,
}

impl RequestDownloadRequest {
    pub(crate) fn new(
        data_format_identifier: DataFormatIdentifier,
        address_and_length_format_identifier: MemoryFormatIdentifier,
        memory_address: u64,
        memory_size: u32,
    ) -> Self {
        assert!(
            memory_address <= 0xFF_FFFF_FFFF,
            "Memory address is too large (max is 0xFF_FFFF_FFFF): {:#X}", memory_address
        );
        Self {
            data_format_identifier,
            address_and_length_format_identifier,
            memory_address,
            memory_size,
        }
    }

    fn get_shortened_memory_address(&self) -> Vec<u8> {
        self.memory_address.to_be_bytes()
            .iter()
            .skip(8 - self.address_and_length_format_identifier.memory_address_length as usize)
            .copied()
            .collect()
    }

    fn get_shortened_memory_size(&self) -> Vec<u8> {
        self.memory_size.to_be_bytes()
            .iter()
            .skip(4 - self.address_and_length_format_identifier.memory_size_length as usize)
            .copied()
            .collect()
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES
    }
}
impl WireFormat for RequestDownloadRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let data_format_identifier = DataFormatIdentifier::from(reader.read_u8()?);
        let memory_identifier = MemoryFormatIdentifier::try_from(reader.read_u8()?)?;

        let mut memory_address: Vec<u8> = vec![0; memory_identifier.memory_address_length as usize];
        let mut memory_size: Vec<u8> = vec![0; memory_identifier.memory_size_length as usize];

        reader.read_exact(&mut memory_address)?;
        reader.read_exact(&mut memory_size)?;

        Ok(Some(Self {
            data_format_identifier,
            address_and_length_format_identifier: memory_identifier,
            memory_address: u64::from_be_bytes({
                let mut bytes = [0; 8];
                bytes[8 - memory_address.len()..].copy_from_slice(&memory_address);
                bytes
            }),
            memory_size: u32::from_be_bytes({
                let mut bytes = [0; 4];
                bytes[4 - memory_size.len()..].copy_from_slice(&memory_size);
                bytes
            }),
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.data_format_identifier.into())?;
        writer.write_u8(self.address_and_length_format_identifier.into())?;

        writer.write_all(self.get_shortened_memory_address().as_mut_slice())?;
        writer.write_all(self.get_shortened_memory_size().as_mut_slice())?;

        Ok(2 + self.address_and_length_format_identifier.len())
    }
}

impl SingleValueWireFormat for RequestDownloadRequest {}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RequestDownloadResponse {
    /// Format is similar to `address_and_length_format_identifier` field of the [`RequestDownloadRequest`] struct.
    /// In it is a byte with the high nibble being the length of the max_number_of_block_length field.
    length_format_identifier: LengthFormatIdentifier,
    /// Variable length field, length determined by `length_format_identifier`
    /// Client is instructed to send this many bytes per [`TransferDataRequest`] message.
    pub max_number_of_block_length: Vec<u8>,
}

impl RequestDownloadResponse {
    pub(crate) fn new(length_format_identifier: u8, max_number_of_block_length: Vec<u8>) -> Self {
        Self {
            length_format_identifier: LengthFormatIdentifier::from(length_format_identifier),
            max_number_of_block_length,
        }
    }
}

impl WireFormat for RequestDownloadResponse {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let length_format_identifier = LengthFormatIdentifier::from(reader.read_u8()?);

        let mut max_number_of_block_length: Vec<u8> =
            vec![0; length_format_identifier.max_number_of_block_length as usize];
        reader.read_exact(&mut max_number_of_block_length)?;

        Ok(Some(Self {
            length_format_identifier,
            max_number_of_block_length,
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.length_format_identifier.into())?;
        writer.write_all(&self.max_number_of_block_length)?;
        Ok(1 + self.max_number_of_block_length.len())
    }
}

impl SingleValueWireFormat for RequestDownloadResponse {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_request() {
        let bytes: [u8; 7] = [
            0x00, // No compression or encryption
            0x14, // 1 byte for memory size, 4 bytes for memory address
            0xF0, 0xFF, 0xFF, 0x67, // memory address
            0x0A,
        ];
        let req = RequestDownloadRequest::option_from_reader(&mut &bytes[..])
            .unwrap()
            .unwrap();
        assert_eq!(u8::from(req.data_format_identifier), 0);
        assert_eq!(u8::from(req.address_and_length_format_identifier), 0x14);
        assert_eq!(req.address_and_length_format_identifier.memory_size_length, 1);
        assert_eq!(req.address_and_length_format_identifier.memory_address_length, 4);

        assert_eq!(req.memory_address, 0xF0FFFF67);
        assert_eq!(req.memory_size, 0x0A);

        assert_eq!(req.get_shortened_memory_address(), vec![0xF0, 0xFF, 0xFF, 0x67]);
        assert_eq!(req.get_shortened_memory_size(), vec![0x0A]);

    }

    #[test]
    fn bad_request() {
        let bytes: [u8; 3] = [
            0x00, // No compression or encryption
            0x11, // 1 byte for memory size, 1 byte for memory address
            0x67,
        ];
        let req = RequestDownloadRequest::option_from_reader(&mut &bytes[..]);
        assert!(matches!(req, Err(Error::IoError(_))));
    }

    #[test]
    fn read_memory_identifier() {
        let memory_format_identifier = MemoryFormatIdentifier::try_from(0x23).unwrap();
        assert_eq!(memory_format_identifier.memory_size_length, 2);
        assert_eq!(memory_format_identifier.memory_address_length, 3);

        assert_eq!(u8::from(memory_format_identifier), 0x23);
    }

    #[test]
    fn read_length_identifier() {
        let length_format_identifier = LengthFormatIdentifier::from(0xF0);
        assert_eq!(length_format_identifier.max_number_of_block_length, 15);

        assert_eq!(u8::from(length_format_identifier), 0xF0);
    }
}
