use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{Error, SingleValueWireFormat, WireFormat};

const MEMORY_SIZE_NIBBLE_MASK:    u8 = 0b1111_0000;
const MEMORY_ADDRESS_NIBBLE_MASK: u8 = 0b0000_1111;

/// Flags byte in the SD protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryFormatIdentifier {
    memory_size: u8,
    memory_address: u8,
}

impl From<u8> for MemoryFormatIdentifier {
    /// Only the two most significant bits are used.
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
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RequestDownloadRequest {
    /// compression method (high nibble) and encrypting method (low nibble). 0x00 is no compression or encryption
    pub data_format_identifier: u8,
    /// 7-4: length (# of bytes) of memory_size param, 3-0: length (# of bytes) of memory_address param
    pub address_and_length_format_identifier: MemoryFormatIdentifier,
    /// 3 bytes. Starting address of the server memory 
    pub memory_address: Vec<u8>,
    /// 3 bytes. 
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
