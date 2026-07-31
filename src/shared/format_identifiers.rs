use crate::Error;

const LOW_NIBBLE_MASK: u8 = 0b0000_1111;
const HIGH_NIBBLE_MASK: u8 = 0b1111_0000;

/// Largest value that fits in a single nibble.
const NIBBLE_MAX: u8 = 0x0F;

/// Address and length format identifier
const MEMORY_SIZE_NIBBLE_MASK: u8 = HIGH_NIBBLE_MASK;
const MEMORY_ADDRESS_NIBBLE_MASK: u8 = LOW_NIBBLE_MASK;

/// Widest `memorySize` the `addressAndLengthFormatIdentifier` may declare, in bytes.
///
/// ISO 14229-1:2020 Annex H Table H.1 marks high-nibble values 1 through 4 applicable
/// (manageable size 256 bytes through 4 GB) and everything else "not applicable".
pub(crate) const MAX_MEMORY_SIZE_LENGTH: u8 = 4;

/// Widest `memoryAddress` the `addressAndLengthFormatIdentifier` may declare, in bytes.
///
/// Table H.1 marks low-nibble values 1 through 5 applicable (addressable memory 256 bytes
/// through 1024 GB - 1).
pub(crate) const MAX_MEMORY_ADDRESS_LENGTH: u8 = 5;

/// Length format identifier
const BLOCK_LENGTH_NIBBLE_MASK: u8 = HIGH_NIBBLE_MASK;

/// Data format identifier
const COMPRESSION_NIBBLE_MASK: u8 = HIGH_NIBBLE_MASK;
const ENCRYPTION_NIBBLE_MASK: u8 = LOW_NIBBLE_MASK;

/// How many bytes the `memoryAddress` and `memorySize` parameters occupy on the wire.
///
/// The `addressAndLengthFormatIdentifier` byte of
/// [`RequestDownloadRequest`](crate::RequestDownloadRequest) and
/// [`RequestUploadRequest`](crate::RequestUploadRequest), which share one message layout.
/// The high nibble is the `memorySize` width and the low nibble the `memoryAddress` width, in
/// bytes — so `0x44` declares four bytes each, and `0x12` one byte of size and two of address.
///
/// ISO 14229-1:2020 Table 441 makes this a client choice rather than a function of the values, so
/// it is a parameter rather than something derived: clause 11.3.1 blesses "a **fixed**
/// addressAndLengthFormatIdentifier" with the unused bytes "padded with the value 0x00", and the
/// standard's own examples are non-minimal (Table 462 declares three `memorySize` bytes for
/// `0x00FFFF`, which needs two). Pass one to
/// [`RequestDownloadRequest::new_with_alfid`](crate::RequestDownloadRequest::new_with_alfid);
/// [`new`](crate::RequestDownloadRequest::new) derives the minimal one for you.
///
/// A bootloader that mandates a particular value states it as a byte, so build one the same way:
///
/// ```
/// use uds_protocol::AddressAndLengthFormatIdentifier as Alfid;
///
/// let alfid = Alfid::try_from(0x44).expect("four bytes each is within Table H.1");
/// assert_eq!(alfid.memory_size_length(), 4);
/// assert_eq!(alfid.memory_address_length(), 4);
/// assert_eq!(u8::from(alfid), 0x44);
///
/// // Annex H Table H.1 marks a zero nibble, and any size above 4, "not applicable".
/// assert!(Alfid::try_from(0x40).is_err());
/// assert!(Alfid::try_from(0x54).is_err());
/// ```
///
/// See ISO 14229-1:2020 Annex H Table H.1 for the applicable widths.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressAndLengthFormatIdentifier {
    /// Width of `memorySize` in bytes: 1 to 4. Private so it cannot leave Table H.1's range.
    memory_size_length: u8,
    /// Width of `memoryAddress` in bytes: 1 to 5. Private for the same reason.
    memory_address_length: u8,
}

impl AddressAndLengthFormatIdentifier {
    /// Build one from its two widths, in bytes.
    ///
    /// Arguments are in **wire order**: `memory_size_length` is the high nibble,
    /// `memory_address_length` the low nibble. Both are `u8`, so the compiler cannot catch a
    /// transposition — if you have the byte already, prefer `try_from`, which cannot be
    /// transposed.
    ///
    /// # Errors
    /// Returns [`Error::InvalidAddressAndLengthFormatIdentifier`] carrying the byte the widths
    /// would have formed if `memory_size_length` is outside 1 to 4 or `memory_address_length` is
    /// outside 1 to 5, which Annex H Table H.1 marks "not applicable".
    pub const fn new(memory_size_length: u8, memory_address_length: u8) -> Result<Self, Error> {
        if !matches!(memory_size_length, 1..=MAX_MEMORY_SIZE_LENGTH)
            || !matches!(memory_address_length, 1..=MAX_MEMORY_ADDRESS_LENGTH)
        {
            // Report the byte, not the nibble, so the message matches what a tester configures
            // and what `try_from` would have been given. Nibbles wider than 4 bits are masked to
            // keep the reported byte a faithful packing of the two.
            return Err(Error::InvalidAddressAndLengthFormatIdentifier(
                ((memory_size_length & LOW_NIBBLE_MASK) << 4)
                    | (memory_address_length & LOW_NIBBLE_MASK),
            ));
        }
        Ok(Self {
            memory_size_length,
            memory_address_length,
        })
    }

    /// Width of the `memorySize` parameter in bytes, 1 to 4. The high nibble.
    #[must_use]
    pub const fn memory_size_length(self) -> u8 {
        self.memory_size_length
    }

    /// Width of the `memoryAddress` parameter in bytes, 1 to 5. The low nibble.
    #[must_use]
    pub const fn memory_address_length(self) -> u8 {
        self.memory_address_length
    }
}

impl PartialEq<u8> for AddressAndLengthFormatIdentifier {
    /// Wire equality: compares the packed byte, so `alfid == 0x44` reads as it does in the spec.
    fn eq(&self, other: &u8) -> bool {
        (self.memory_size_length << 4) | self.memory_address_length == *other
    }
}

impl TryFrom<u8> for AddressAndLengthFormatIdentifier {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        // High nibble: bytes used for the memorySize parameter. Table H.1 marks 1 through 4
        // applicable (manageable size 256 bytes to 4 GB); 0 and 5..=15 are "not applicable".
        let memory_size_length = (value & MEMORY_SIZE_NIBBLE_MASK) >> 4;
        // Low nibble: bytes used for the memoryAddress parameter. Table H.1 marks 1 through 5
        // applicable (addressable memory 256 bytes to 1024 GB - 1).
        let memory_address_length = value & MEMORY_ADDRESS_NIBBLE_MASK;

        if !matches!(memory_size_length, 1..=MAX_MEMORY_SIZE_LENGTH) {
            return Err(Error::InvalidAddressAndLengthFormatIdentifier(value));
        }
        if !matches!(memory_address_length, 1..=MAX_MEMORY_ADDRESS_LENGTH) {
            return Err(Error::InvalidAddressAndLengthFormatIdentifier(value));
        }
        Ok(Self {
            memory_size_length,
            memory_address_length,
        })
    }
}

impl From<AddressAndLengthFormatIdentifier> for u8 {
    fn from(alfid: AddressAndLengthFormatIdentifier) -> u8 {
        (alfid.memory_size_length << 4) | alfid.memory_address_length
    }
}

/// The leading byte of a [`RequestDownloadResponse`](crate::RequestDownloadResponse) or
/// [`RequestUploadResponse`](crate::RequestUploadResponse), which share one message layout.
///
/// The format mirrors [`AddressAndLengthFormatIdentifier`]: a byte whose high nibble gives the byte
/// length of `max_number_of_block_length`, i.e. `0x20` means that field is 2 bytes long.
/// Derived from the slice length when encoding, so it is not part of either response's
/// public surface.
///
/// The low nibble is not retained. ISO 14229-1:2020 Tables 443 and 448 both state that bits
/// 3 to 0 are "reserved by document, to be set to '0'", and that "the lower nibble **shall**
/// be set to '0'" — the byte range they give is `0x00` to `0xF0`. A non-zero low nibble is
/// therefore not a value to echo back but a malformed byte, so encoding always emits zero.
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
/// Serializes as the single wire byte it occupies. Going through the byte rather than the two
/// private nibble fields means `serde` cannot mint a nibble wider than `0x0F`, which `new`
/// rejects — the byte form simply has no such state to represent.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(from = "u8", into = "u8"))]
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

    /// The single byte this identifier occupies on the wire.
    ///
    /// `const`, unlike `u8::from(identifier)`, so it can be used in a `const` context.
    #[must_use]
    pub const fn value(&self) -> u8 {
        self.encryption_method | (self.compression_method << 4)
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
        let memory_format_identifier = AddressAndLengthFormatIdentifier::try_from(0x23).unwrap();
        assert_eq!(memory_format_identifier.memory_size_length, 2);
        assert_eq!(memory_format_identifier.memory_address_length, 3);

        assert_eq!(u8::from(memory_format_identifier), 0x23);
    }

    #[test]
    fn failed_memory_format_identifier() {
        // Tables 444 and 449 both put an invalid addressAndLengthFormatIdentifier on NRC 0x31
        // requestOutOfRange, not 0x13.
        let memory_format_identifier = AddressAndLengthFormatIdentifier::try_from(0x00);
        assert!(matches!(
            memory_format_identifier,
            Err(Error::InvalidAddressAndLengthFormatIdentifier(0x00))
        ));
        assert_eq!(
            Error::InvalidAddressAndLengthFormatIdentifier(0x00).negative_response_code(),
            Some(crate::NegativeResponseCode::RequestOutOfRange)
        );
    }

    #[test]
    fn every_alfid_legal_per_table_h1_is_accepted() {
        // ISO 14229-1:2020 Annex H Table H.1 runs from 0x11 to 0x45: the high nibble
        // (memorySize) is applicable for 1..=4 bytes and the low nibble (memoryAddress) for
        // 1..=5. Anything outside that is "not applicable" and must be rejected.
        for size_len in 1..=4u8 {
            for addr_len in 1..=5u8 {
                let byte = (size_len << 4) | addr_len;
                let mfi = AddressAndLengthFormatIdentifier::try_from(byte)
                    .unwrap_or_else(|e| panic!("Table H.1 lists {byte:#04X} as valid, got {e:?}"));
                assert_eq!(mfi.memory_size_length, size_len, "for {byte:#04X}");
                assert_eq!(mfi.memory_address_length, addr_len, "for {byte:#04X}");
                assert_eq!(u8::from(mfi), byte, "round trip for {byte:#04X}");
            }
        }
    }

    #[test]
    fn alfid_nibbles_outside_table_h1_are_rejected() {
        // A zero nibble is "not applicable" on either side, and the widths stop at 4 (size)
        // and 5 (address).
        for byte in [0x00, 0x01, 0x10, 0x05, 0x50, 0x46, 0x55, 0xF5, 0x5F] {
            assert!(
                AddressAndLengthFormatIdentifier::try_from(byte).is_err(),
                "{byte:#04X} is not applicable per Table H.1 but was accepted"
            );
        }
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
            fn prop_length_format_identifier_normalizes_the_reserved_nibble(byte in any::<u8>()) {
                // Tables 443 and 448 require the low nibble to be '0', so re-encoding any byte
                // must clear it while preserving the block-length nibble. Generating the full
                // `u8` range rather than `high_nibble << 4` is what makes this a real property:
                // the narrower generator held trivially whichever way the impl behaved.
                let lfi = LengthFormatIdentifier::from(byte);
                let back: u8 = lfi.into();
                prop_assert_eq!(byte & 0xF0, back);
            }

            #[test]
            fn prop_memory_format_identifier_roundtrip(
                // The full range Annex H Table H.1 declares applicable. Narrowing these to
                // what the decoder happened to accept is what previously let an off-by-one in
                // the range checks sit underneath a passing property test.
                size_len in 1u8..=MAX_MEMORY_SIZE_LENGTH,
                addr_len in 1u8..=MAX_MEMORY_ADDRESS_LENGTH,
            ) {
                let byte = (size_len << 4) | addr_len;
                let mfi = AddressAndLengthFormatIdentifier::try_from(byte).unwrap();
                let back: u8 = mfi.into();
                prop_assert_eq!(byte, back);
            }
        }
    }
}
