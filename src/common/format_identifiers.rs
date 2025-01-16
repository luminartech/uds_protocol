use crate::Error;
use serde::{Deserialize, Serialize};

const LOW_NIBBLE_MASK:  u8 = 0b0000_1111;
const HIGH_NIBBLE_MASK: u8 = 0b1111_0000;

/// Address and length format identifier
const MEMORY_SIZE_NIBBLE_MASK:    u8 = HIGH_NIBBLE_MASK;
const MEMORY_ADDRESS_NIBBLE_MASK: u8 = LOW_NIBBLE_MASK;

/// Length format identifier
const BLOCK_LENGTH_NIBBLE_MASK:   u8 = HIGH_NIBBLE_MASK;

/// Data format identifier
const COMPRESSION_NIBBLE_MASK:   u8 = HIGH_NIBBLE_MASK;
const ENCRYPTION_NIBBLE_MASK:   u8 = LOW_NIBBLE_MASK;


/// Decoded from the `address_and_length_format_identifier` field of the [`RequestDownloadRequest`] struct
/// 
/// See ISO-14229-1:2020, Table H.1 for format information
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MemoryFormatIdentifier {
    pub memory_size_length: u8,
    pub memory_address_length: u8,
}

impl MemoryFormatIdentifier {
    pub fn new(memory_size: u32, memory_address: u64) -> Self {

        let memory_address_length = ((64 - memory_address.leading_zeros() + 7) / 8) as u8;
        let memory_size_length = ((32 - memory_size.leading_zeros() + 7) / 8) as u8;

        Self {
            memory_size_length,
            memory_address_length,
        }
    }
    
    /// Get the total length of the memory_size and memory_address fields 
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.memory_size_length as usize + self.memory_address_length as usize
    }
}

impl TryFrom<u8> for MemoryFormatIdentifier {
    type Error = Error;
    // NRC::RequestOutOfRange if address_and_length_format_identifier is not valid
    fn try_from(value: u8) -> Result<Self, Error> {
        let memory_size_length = (value & MEMORY_SIZE_NIBBLE_MASK) >> 4;
        let memory_address_length = value & MEMORY_ADDRESS_NIBBLE_MASK;

        match memory_size_length {
            1..4 => (),
            _ => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
        }
        match memory_address_length {
            1..5 => (),
            _ => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
        }
        Ok(Self {
            // get the low nibble of address_and_length_format_identifier
            // Memory size length is 1 through 4 bytes (manageable size: 256 bytes to 4GB)
            memory_size_length,
            // get the high nibble of address_and_length_format_identifier
            // Memory address is 1 through 5 bytes (addressable memory: 256 bytes - 1024GB)
            memory_address_length: value & MEMORY_ADDRESS_NIBBLE_MASK,
        })
    }
}

impl From<MemoryFormatIdentifier> for u8 {
    fn from(memory_format_identifier: MemoryFormatIdentifier) -> u8 {
        (memory_format_identifier.memory_size_length << 4) | memory_format_identifier.memory_address_length
    }
}

/// Decoded from the `length_format_identifier` field of the [`RequestDownloadResponse`] struct
/// Format is similar to `address_and_length_format_identifier` field of the [`RequestDownloadRequest`] struct
/// As in it is a nibble with the high nibble being the byte length of the max_number_of_block_length field
/// I.E. 0x20 means the max_number_of_block_length field is 2 bytes long
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LengthFormatIdentifier {
    pub max_number_of_block_length: u8,
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

/// Used by [`RequestDownloadRequest`] for the compression method (high nibble) and encrypting method (low nibble)
/// - 0x00 is no compression or encryption, which is the default
/// 
/// Decoded from the `data_format_identifier` field of the [`RequestDownloadRequest`] struct
/// Values other than 0x00 are Vehicle Manufacturer specific according to ISO-14229-1:2020
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DataFormatIdentifier {
    // low nibble
    pub encryption_method: u8,
    // high nibble
    pub compression_method: u8,
}

impl DataFormatIdentifier {
    pub fn new(encryption_method: u8, compression_method: u8) -> Result<Self, Error> {
        Ok(Self {
            encryption_method: Self::check_value(encryption_method)?,
            compression_method: Self::check_value(compression_method)?,
        })
    }
    fn check_value(value: u8) -> Result<u8, Error> {
        match value {
            0..=15 => Ok(value),
            _ => Err(Error::InvalidEncryptionCompressionMethod(value)),
        }
    }
}
impl From<u8> for DataFormatIdentifier {
    fn from(value: u8) -> Self {

        let encryption_method = value & ENCRYPTION_NIBBLE_MASK;
        let compression_method = (value & COMPRESSION_NIBBLE_MASK) >> 4;
        
        Self {
            encryption_method,
            compression_method,
        }
    }
}
impl From<DataFormatIdentifier> for u8 {
    fn from(data_format_identifier: DataFormatIdentifier) -> u8 {
        data_format_identifier.encryption_method | data_format_identifier.compression_method << 4
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn memory_format_identifier() {
        let memory_format_identifier = MemoryFormatIdentifier::try_from(0x23).unwrap();
        assert_eq!(memory_format_identifier.memory_size_length, 2);
        assert_eq!(memory_format_identifier.memory_address_length, 3);

        assert_eq!(u8::from(memory_format_identifier), 0x23);
    }

    #[test]
    fn failed_memory_format_identifier() {
        let memory_format_identifier = MemoryFormatIdentifier::try_from(0x00);
        assert!(matches!(memory_format_identifier, Err(Error::IncorrectMessageLengthOrInvalidFormat)));
    }

    #[test]
    fn length_format_identifier() {
        let length_format_identifier = LengthFormatIdentifier::from(0xF0);
        assert_eq!(length_format_identifier.max_number_of_block_length, 15);

        assert_eq!(u8::from(length_format_identifier), 0xF0);
    }

    #[test]
    fn data_format_identifier() {
        let data_format_identifier = DataFormatIdentifier::try_from(0x23).unwrap();
        assert_eq!(data_format_identifier.encryption_method, 3);
        assert_eq!(data_format_identifier.compression_method, 2);

        assert_eq!(u8::from(data_format_identifier), 0x23);

        let data_format_identifier = DataFormatIdentifier::new(0x0F, 0x0F);
        assert!(matches!(data_format_identifier, Ok(_)));

        let data_format_identifier = DataFormatIdentifier::new(0x1F, 0x0F);
        assert!(matches!(data_format_identifier, Err(Error::IncorrectMessageLengthOrInvalidFormat)));
    }

}