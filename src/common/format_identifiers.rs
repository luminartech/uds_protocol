use crate::{Error, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};

const LOW_NIBBLE_MASK: u8 = 0b0000_1111;
const HIGH_NIBBLE_MASK: u8 = 0b1111_0000;

/// Address and length format identifier
const MEMORY_SIZE_NIBBLE_MASK: u8 = HIGH_NIBBLE_MASK;
const MEMORY_ADDRESS_NIBBLE_MASK: u8 = LOW_NIBBLE_MASK;

/// Length format identifier
const BLOCK_LENGTH_NIBBLE_MASK: u8 = HIGH_NIBBLE_MASK;

/// Data format identifier
const COMPRESSION_NIBBLE_MASK: u8 = HIGH_NIBBLE_MASK;
const ENCRYPTION_NIBBLE_MASK: u8 = LOW_NIBBLE_MASK;

/// Takes in the actual memory address to be used and the size of the memory to be used
/// and computes how many bytes are needed to represent them
///
/// Decoded from the `address_and_length_format_identifier` field of the [`crate::RequestDownloadRequest`] struct
///
/// See ISO-14229-1:2020, Table H.1 for format information
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MemoryFormatIdentifier {
    pub memory_size_length: u8,
    pub memory_address_length: u8,
}

impl MemoryFormatIdentifier {
    /// Takes in the actual memory address to be used and the size of the memory to be used
    /// and computes how many bytes are needed to represent them
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_values(memory_size: u32, memory_address: u64) -> Self {
        let memory_address_length = (u64::BITS - memory_address.leading_zeros()).div_ceil(8) as u8;
        let memory_size_length = (u32::BITS - memory_size.leading_zeros()).div_ceil(8) as u8;

        Self {
            memory_size_length,
            memory_address_length,
        }
    }

    /// Get the total length of the `memory_size` and `memory_address` fields
    pub fn len(self) -> usize {
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
        (memory_format_identifier.memory_size_length << 4)
            | memory_format_identifier.memory_address_length
    }
}

/// Decoded from the `length_format_identifier` field of the [`RequestDownloadResponse`] struct.
/// The format is similar to the `address_and_length_format_identifier` field in the [`RequestDownloadRequest`] struct.
/// Specifically, it is a byte where the high nibble represents the byte length of the `max_number_of_block_length` field,
/// i.e, a value of `0x20` indicates that the `max_number_of_block_length` field is 2 bytes long.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LengthFormatIdentifier {
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

/// Used by [`crate::RequestDownloadRequest`] for the compression method (high nibble) and encrypting method (low nibble)
/// - 0x00 is no compression or encryption, which is the default
///
/// Decoded from the `data_format_identifier` field of the [`crate::RequestDownloadRequest`] struct
/// Values other than 0x00 are Vehicle Manufacturer specific according to ISO-14229-1:2020
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataFormatIdentifier {
    // low nibble
    encryption_method: u8,
    // high nibble
    compression_method: u8,
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
        data_format_identifier.encryption_method | (data_format_identifier.compression_method << 4)
    }
}

// compare to a u8 value
impl PartialEq<u8> for DataFormatIdentifier {
    fn eq(&self, other: &u8) -> bool {
        let other_data_format_identifier = DataFormatIdentifier::from(*other);
        self == &other_data_format_identifier
    }
}

impl WireFormat for DataFormatIdentifier {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value = reader.read_u8()?;
        Ok(Some(DataFormatIdentifier::from(value)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(*self))?;
        Ok(1)
    }
}

impl SingleValueWireFormat for DataFormatIdentifier {}

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
        assert!(matches!(
            memory_format_identifier,
            Err(Error::IncorrectMessageLengthOrInvalidFormat)
        ));
    }

    #[test]
    fn length_format_identifier() {
        let length_format_identifier = LengthFormatIdentifier::from(0xF0);
        assert_eq!(length_format_identifier.max_number_of_block_length, 15);

        assert_eq!(u8::from(length_format_identifier), 0xF0);
    }

    #[test]
    fn data_format_identifier() {
        let data_format_identifier = DataFormatIdentifier::from(0x23);
        assert_eq!(data_format_identifier.encryption_method, 3);
        assert_eq!(data_format_identifier.compression_method, 2);

        assert_eq!(u8::from(data_format_identifier), 0x23);

        let data_format_identifier = DataFormatIdentifier::new(0x0F, 0x0F);
        assert!(data_format_identifier.is_ok());

        let data_format_identifier = DataFormatIdentifier::new(0x1F, 0x0F);
        assert!(matches!(
            data_format_identifier,
            Err(Error::InvalidEncryptionCompressionMethod(0x1F))
        ));
    }
}
