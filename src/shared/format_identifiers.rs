use crate::Error;

const LOW_NIBBLE_MASK: u8 = 0b0000_1111;
const HIGH_NIBBLE_MASK: u8 = 0b1111_0000;

/// Largest value that fits in a single nibble.
const NIBBLE_MAX: u8 = 0x0F;

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
/// Carried by the `addressAndLengthFormatIdentifier` byte of
/// [`RequestDownloadRequest`](crate::RequestDownloadRequest) and
/// [`RequestUploadRequest`](crate::RequestUploadRequest), which share one message layout.
/// Derived from the address and size rather than set by the caller, so it is not part of
/// either type's public surface.
///
/// See ISO-14229-1:2020, Table H.1 for format information
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MemoryFormatIdentifier {
    pub memory_size_length: u8,
    pub memory_address_length: u8,
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

/// The leading byte of a [`RequestDownloadResponse`](crate::RequestDownloadResponse) or
/// [`RequestUploadResponse`](crate::RequestUploadResponse), which share one message layout.
///
/// The format mirrors [`MemoryFormatIdentifier`]: a byte whose high nibble gives the byte
/// length of `max_number_of_block_length`, i.e. `0x20` means that field is 2 bytes long.
/// Derived from the slice length when encoding, so it is not part of either response's
/// public surface.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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

/// The compression method (high nibble) and encryption method (low nibble) a client asks the
/// server to use for a transfer.
///
/// - `0x00` for both means no compression and no encryption, which is the default; prefer
///   [`DataFormatIdentifier::NONE`].
/// - Values other than `0x00` are Vehicle Manufacturer specific according to ISO-14229-1:2020.
///
/// Supplied to [`RequestDownloadRequest::new`](crate::RequestDownloadRequest::new) and
/// [`RequestUploadRequest::new`](crate::RequestUploadRequest::new), and read back with
/// [`RequestDownloadRequest::data_format_identifier`](crate::RequestDownloadRequest::data_format_identifier).
/// Also carried by the `AddFile`, `ReplaceFile`, `ReadFile` and `ResumeFile` variants of
/// [`RequestFileTransferRequest`](crate::RequestFileTransferRequest).
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataFormatIdentifier {
    // low nibble
    encryption_method: u8,
    // high nibble
    compression_method: u8,
}

impl DataFormatIdentifier {
    /// No compression and no encryption — the `0x00` byte, and the overwhelmingly common case.
    pub const NONE: Self = Self {
        compression_method: 0,
        encryption_method: 0,
    };

    /// Build a `DataFormatIdentifier` from its compression and encryption method nibbles.
    ///
    /// Arguments are in **wire order**: compression is the high nibble, encryption the low
    /// nibble. `0x00` for both means no compression and no encryption — prefer
    /// [`DataFormatIdentifier::NONE`] for that. Each value occupies a single nibble.
    ///
    /// Both parameters are `u8`, so the compiler cannot catch a transposition; if you are
    /// converting a byte you already have, use `DataFormatIdentifier::from(byte)` instead.
    ///
    /// # Errors
    /// Returns [`Error::InvalidEncryptionCompressionMethod`] if either value does not fit
    /// in a nibble (i.e. is greater than `0x0F`).
    // Written as explicit range checks rather than a `?` on a helper: `?` is not permitted in
    // a `const fn`, and const construction is what lets callers put a `DataFormatIdentifier`
    // in a `const` table.
    pub const fn new(compression_method: u8, encryption_method: u8) -> Result<Self, Error> {
        if compression_method > NIBBLE_MAX {
            return Err(Error::InvalidEncryptionCompressionMethod(
                compression_method,
            ));
        }
        if encryption_method > NIBBLE_MAX {
            return Err(Error::InvalidEncryptionCompressionMethod(encryption_method));
        }
        Ok(Self {
            encryption_method,
            compression_method,
        })
    }

    /// The compression method nibble (the high nibble on the wire).
    ///
    /// `0x00` means no compression. Other values are vehicle-manufacturer specific.
    #[must_use]
    pub const fn compression_method(&self) -> u8 {
        self.compression_method
    }

    /// The encryption method nibble (the low nibble on the wire).
    ///
    /// `0x00` means no encryption. Other values are vehicle-manufacturer specific.
    #[must_use]
    pub const fn encryption_method(&self) -> u8 {
        self.encryption_method
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

        // Arguments are in wire order: compression is the high nibble.
        let dfi = DataFormatIdentifier::new(0x02, 0x01).unwrap();
        assert_eq!(dfi.compression_method(), 0x02);
        assert_eq!(dfi.encryption_method(), 0x01);
        assert_eq!(
            u8::from(dfi),
            0x21,
            "compression must land in the high nibble"
        );

        assert_eq!(u8::from(DataFormatIdentifier::NONE), 0x00);
        assert_eq!(DataFormatIdentifier::NONE, DataFormatIdentifier::from(0x00));
    }

    mod prop {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_data_format_identifier_roundtrip(byte in any::<u8>()) {
                let dfi = DataFormatIdentifier::from(byte);
                let back: u8 = dfi.into();
                prop_assert_eq!(byte, back);
            }

            #[test]
            fn prop_length_format_identifier_roundtrip(high_nibble in 0u8..=15) {
                // LengthFormatIdentifier only stores the high nibble
                let byte = high_nibble << 4;
                let lfi = LengthFormatIdentifier::from(byte);
                let back: u8 = lfi.into();
                prop_assert_eq!(byte, back);
            }

            #[test]
            fn prop_memory_format_identifier_roundtrip(
                size_len in 1u8..=3,
                addr_len in 1u8..=4,
            ) {
                let byte = (size_len << 4) | addr_len;
                let mfi = MemoryFormatIdentifier::try_from(byte).unwrap();
                let back: u8 = mfi.into();
                prop_assert_eq!(byte, back);
            }
        }
    }
}
