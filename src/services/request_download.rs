use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{Error, NegativeResponseCode, SingleValueWireFormat, WireFormat};

const MEMORY_SIZE_NIBBLE_MASK:    u8 = 0b1111_0000;
const MEMORY_ADDRESS_NIBBLE_MASK: u8 = 0b0000_1111;

const BLOCK_LENGTH_NIBBLE_MASK:   u8 = 0b1111_0000;

const REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 6] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::AuthenticationRequired,
    NegativeResponseCode::UploadDownloadNotAccepted,
];
/// Decoded from the `address_and_length_format_identifier` field of the [`RequestDownloadRequest`] struct
/// 
/// See ISO-14229-1:2020, Table H.1 for format information
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryFormatIdentifier {
    memory_size: u8,
    memory_address: u8,
}

impl From<u8> for MemoryFormatIdentifier {
    // NRC::RequestOutOfRange if address_and_length_format_identifier is not valid
    fn from(value: u8) -> Self {
        Self {
            // get the low nibble of address_and_length_format_identifier
            // Memory size length is 1 through 4 bytes (manageable size: 256 bytes to 4GB)
            memory_size: (value & MEMORY_SIZE_NIBBLE_MASK) >> 4,
            // get the high nibble of address_and_length_format_identifier
            // Memory address is 1 through 5 bytes (addressable memory: 256 bytes - 1024GB)
            memory_address: value & MEMORY_ADDRESS_NIBBLE_MASK,
        }
    }
}
impl From<MemoryFormatIdentifier> for u8 {
    fn from(memory_format_identifier: MemoryFormatIdentifier) -> u8 {
        (memory_format_identifier.memory_size << 4) | memory_format_identifier.memory_address
    }
}

/// A request to the server for it to download data from the client
/// 
/// A positive response to this request ([`RequestDownloadResponse`]) will happen 
/// after the server takes all necessary actions to receive the data 
/// (n.b. not sure if this is AFTER the data is received or just once the server is READY to receive)
/// 
/// This is a variable length Request, determined by the `address_and_length_format_identifier` value
/// See ISO-14229-1:2020, Table H.1 for format information
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RequestDownloadRequest {
    /// compression method (high nibble) and encrypting method (low nibble). 0x00 is no compression or encryption
    pub data_format_identifier: u8,
    /// 7-4: length (# of bytes) of memory_size param, 3-0: length (# of bytes) of memory_address param
    pub address_and_length_format_identifier: MemoryFormatIdentifier,
    /// Starting address of the server memory. Size is determined by `address_and_length_format_identifier`
    pub memory_address: Vec<u8>,
    /// Size of the data to be downloaded. Number of bytes sent is determined by `address_and_length_format_identifier`
    /// Used by the server to validate the data transferred by the [`TransferData`] service
    pub memory_size: Vec<u8>,
}

impl RequestDownloadRequest {
    pub(crate) fn new(
        data_format_identifier: u8,
        address_and_length_format_identifier: u8,
        memory_address: Vec<u8>,
        memory_size: Vec<u8>,
    ) -> Self {
        Self {
            data_format_identifier,
            address_and_length_format_identifier: MemoryFormatIdentifier::from(address_and_length_format_identifier),
            memory_address,
            memory_size,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES
    }
}
impl WireFormat for RequestDownloadRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let data_format_identifier = reader.read_u8()?;
        let address_and_length_format_identifier = MemoryFormatIdentifier::from(reader.read_u8()?);

        let mut memory_address: Vec<u8> = vec![0; address_and_length_format_identifier.memory_address as usize];
        let mut memory_size: Vec<u8> = vec![0; address_and_length_format_identifier.memory_size as usize];

        // Read u8's until we have the correct number of bytes for memory_address
        reader.read_exact(&mut memory_address)?;
        reader.read_exact(&mut memory_size)?;

        Ok(Some(Self {
            data_format_identifier,
            address_and_length_format_identifier,
            memory_address,
            memory_size,
        }))
    }
    
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.data_format_identifier)?;
        writer.write_u8(self.address_and_length_format_identifier.into())?;
        writer.write_all(&self.memory_address)?;
        writer.write_all(&self.memory_size)?;
        Ok(2 + self.memory_address.len() + self.memory_size.len())
    }
}

impl SingleValueWireFormat for RequestDownloadRequest {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LengthFormatIdentifier {
    max_number_of_block_length: u8,
}

impl From<u8> for LengthFormatIdentifier {
    fn from(value: u8) -> Self {
        Self {
            max_number_of_block_length: (value & BLOCK_LENGTH_NIBBLE_MASK) >> 4,
        }
    }
}
impl From<LengthFormatIdentifier> for u8 {
    fn from(length_format_identifier: LengthFormatIdentifier) -> u8 {
        length_format_identifier.max_number_of_block_length << 4
    }
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RequestDownloadResponse {
    /// Format is similar to `address_and_length_format_identifier` field of the [`RequestDownloadRequest`] struct
    /// As in it is a nibble with the high nibble being the length of the max_number_of_block_length field
    pub length_format_identifier: LengthFormatIdentifier,
    /// Variable length field, length determined by `length_format_identifier`
    /// Client is instructed to send this many bytes per [`TransferDataRequest`] message.
    pub max_number_of_block_length: Vec<u8>,
}

impl RequestDownloadResponse {
    pub(crate) fn new(
        length_format_identifier: u8,
        max_number_of_block_length: Vec<u8>,
    ) -> Self {
        Self { 
            length_format_identifier: LengthFormatIdentifier::from(length_format_identifier),
            max_number_of_block_length
        }
    }
}

impl WireFormat for RequestDownloadResponse {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let length_format_identifier = LengthFormatIdentifier::from(reader.read_u8()?);
        
        let mut max_number_of_block_length: Vec<u8> = vec![0; length_format_identifier.max_number_of_block_length as usize];
        reader.read_exact(&mut max_number_of_block_length)?;

        Ok(Some(Self {
            length_format_identifier,
            max_number_of_block_length
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
        let bytes: [u8; 4] = [
            0x00, // No compression or encryption
            0x11, // 1 byte for memory size, 1 byte for memory address
            0x67, 
            10
        ];
        let request_download_request = 
            RequestDownloadRequest::option_from_reader(&mut &bytes[..]).unwrap().unwrap();
        assert_eq!(request_download_request.data_format_identifier, 0);
        assert_eq!(u8::from(request_download_request.address_and_length_format_identifier), 0x11);
        assert_eq!(request_download_request.memory_address, vec![0x67]);
        assert_eq!(request_download_request.memory_size, vec![0x0A]);
    }

    #[test]
    fn bad_request() {
        let bytes: [u8; 3] = [
            0x00, // No compression or encryption
            0x11, // 1 byte for memory size, 1 byte for memory address
            0x67, 
        ];
        let request_download_request = RequestDownloadRequest::option_from_reader(&mut &bytes[..]);
        assert!(matches!(request_download_request, Err(Error::IoError(_))));
    }

    #[test]
    fn read_memory_identifier() {
        let memory_format_identifier = MemoryFormatIdentifier::from(0x23);
        assert_eq!(memory_format_identifier.memory_size, 2);
        assert_eq!(memory_format_identifier.memory_address, 3);

        assert_eq!(u8::from(memory_format_identifier), 0x23);
    }

    #[test]
    fn read_length_identifier() {
        let length_format_identifier = LengthFormatIdentifier::from(0xF0);
        assert_eq!(length_format_identifier.max_number_of_block_length, 15);

        assert_eq!(u8::from(length_format_identifier), 0xF0);
    }
}
