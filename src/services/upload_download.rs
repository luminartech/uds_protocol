//! `RequestDownload` (0x34) and `RequestUpload` (0x35) service implementations.
//!
//! ISO 14229-1 gives the two services identical message layouts — request:
//! `dataFormatIdentifier`, `addressAndLengthFormatIdentifier`, `memoryAddress`, `memorySize`;
//! positive response: `lengthFormatIdentifier`, `maxNumberOfBlockLength` — differing only in
//! service identifier and in which direction the subsequent `TransferData` sequence moves the
//! bytes. Both pairs are generated from one macro so the wire codec has a single source of
//! truth; a fix to the width-derivation logic cannot land on one service and miss the other.

use crate::shared::{
    DataFormatIdentifier, LengthFormatIdentifier, MAX_MEMORY_ADDRESS_LENGTH,
    MAX_MEMORY_SIZE_LENGTH, MemoryFormatIdentifier,
};
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};
use automotive_wire_codec::{read_be_uint_into, write_all, write_be_uint, write_u8};

/// Widest `maxNumberOfBlockLength` a `lengthFormatIdentifier` nibble can declare, in bytes.
const MAX_BLOCK_LENGTH_BYTES: usize = 0x0F;

/// Permitted NRCs for `RequestDownload` (0x34).
const REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 6] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::AuthenticationRequired,
    NegativeResponseCode::UploadDownloadNotAccepted,
];

/// Permitted NRCs for `RequestUpload` (0x35). ISO 14229-1 specifies the same set as
/// `RequestDownload`; kept as a separate constant so either service can diverge later without
/// silently changing the other.
const REQUEST_UPLOAD_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 6] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::AuthenticationRequired,
    NegativeResponseCode::UploadDownloadNotAccepted,
];

macro_rules! upload_download_service {
    (
        request: $req:ident,
        request_serde_repr: $repr:ident,
        request_serde_repr_name: $repr_str:literal,
        response: $resp:ident,
        nrcs: $nrcs:ident,
        request_doc: $req_doc:literal,
        response_doc: $resp_doc:literal,
        verb: $verb:literal,
        tests: $test_mod:ident,
    ) => {
        #[doc = $req_doc]
        ///
        /// This is a variable length request, determined by the
        /// `address_and_length_format_identifier` value.
        /// See ISO-14229-1:2020, Table H.1 for format information.
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "serde", serde(try_from = $repr_str, into = $repr_str))]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub struct $req {
            /// compression method (high nibble) and encrypting method (low nibble). 0x00 is no
            /// compression or encryption
            data_format_identifier: DataFormatIdentifier,
            /// 7-4: length (# of bytes) of `memory_size` param, 3-0: length (# of bytes) of
            /// `memory_address` param
            address_and_length_format_identifier: MemoryFormatIdentifier,
            /// Starting address of the server memory. The on-wire byte width is derived from
            /// this value (max 5 bytes), so it is private to keep it in sync with the format
            /// identifier.
            memory_address: u64,
            #[doc = concat!("Size of the data to be ", $verb, ". The on-wire byte width is")]
            /// derived from this value (max 4 bytes), so it is private to keep it in sync with
            /// the format identifier.
            memory_size: u32,
        }

        impl $req {
            #[doc = concat!("Create a new `", stringify!($req), "`")]
            ///
            /// # Errors
            /// Returns an error if `memory_address` exceeds 5 bytes (> `0xFF_FFFF_FFFF`).
            #[allow(clippy::cast_possible_truncation)]
            pub const fn new(
                data_format_identifier: DataFormatIdentifier,
                memory_address: u64,
                memory_size: u32,
            ) -> Result<Self, Error> {
                if memory_address > 0xFF_FFFF_FFFF {
                    return Err(Error::InvalidMemoryAddress(memory_address));
                }
                // A length of 0 produces an invalid `MemoryFormatIdentifier` (the nibbles
                // must be >=1 per ISO-14229), so clamp to at least one byte even when the
                // address or size is 0. Written as `if` rather than `.max(1)` because
                // `Ord::max` is not callable in a `const fn`.
                let address_bytes = (u64::BITS - memory_address.leading_zeros()).div_ceil(8) as u8;
                let memory_address_length = if address_bytes == 0 { 1 } else { address_bytes };
                let size_bytes = (u32::BITS - memory_size.leading_zeros()).div_ceil(8) as u8;
                let memory_size_length = if size_bytes == 0 { 1 } else { size_bytes };
                let address_and_length_format_identifier = MemoryFormatIdentifier {
                    memory_size_length,
                    memory_address_length,
                };
                Ok(Self {
                    data_format_identifier,
                    address_and_length_format_identifier,
                    memory_address,
                    memory_size,
                })
            }

            #[doc = concat!("Create a `", stringify!($req), "` with client-chosen field widths.")]
            ///
            /// ISO 14229-1:2020 Table 441 makes the `addressAndLengthFormatIdentifier` a
            /// client choice rather than a function of the values, and many bootloaders
            /// require a particular one (often `0x44`) and answer `requestOutOfRange`
            /// otherwise. [`new`](Self::new) always derives the *minimal* widths, so it cannot
            /// express a wider-than-minimal declaration; this can.
            ///
            /// Widths are in bytes: `memory_address_length` may be 1 through 5 and
            /// `memory_size_length` 1 through 4 (Annex H Table H.1).
            ///
            /// # Errors
            /// Returns [`Error::IncorrectMessageLengthOrInvalidFormat`] if either width is
            /// outside the range Table H.1 permits, or if a value does not fit in the width
            /// declared for it — which would otherwise truncate it on the wire.
            pub const fn new_with_widths(
                data_format_identifier: DataFormatIdentifier,
                memory_address: u64,
                memory_address_length: u8,
                memory_size: u32,
                memory_size_length: u8,
            ) -> Result<Self, Error> {
                if memory_address_length < 1
                    || memory_address_length > MAX_MEMORY_ADDRESS_LENGTH
                    || memory_size_length < 1
                    || memory_size_length > MAX_MEMORY_SIZE_LENGTH
                {
                    return Err(Error::IncorrectMessageLengthOrInvalidFormat);
                }
                // A width of n bytes holds values below 1 << (8 * n). Shifting is done on u128
                // so the 8-byte case cannot overflow, and compared in u128 for the same reason.
                if (memory_address as u128) >= (1u128 << (8 * memory_address_length as u32))
                    || (memory_size as u128) >= (1u128 << (8 * memory_size_length as u32))
                {
                    return Err(Error::IncorrectMessageLengthOrInvalidFormat);
                }
                Ok(Self {
                    data_format_identifier,
                    address_and_length_format_identifier: MemoryFormatIdentifier {
                        memory_size_length,
                        memory_address_length,
                    },
                    memory_address,
                    memory_size,
                })
            }

            /// The compression and encryption methods the client asked the server to use.
            ///
            /// A server has to act on this, so it must be readable back off a decoded request;
            /// use [`DataFormatIdentifier::compression_method`] and
            /// [`DataFormatIdentifier::encryption_method`] for the individual nibbles.
            #[must_use]
            pub const fn data_format_identifier(&self) -> DataFormatIdentifier {
                self.data_format_identifier
            }

            /// Starting address of the server memory.
            #[must_use]
            pub const fn memory_address(&self) -> u64 {
                self.memory_address
            }

            #[doc = concat!("Size of the data to be ", $verb, ".")]
            #[must_use]
            pub const fn memory_size(&self) -> u32 {
                self.memory_size
            }

            /// Get the allowed [`NegativeResponseCode`] variants for this request
            #[must_use]
            pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
                &$nrcs
            }
        }

        /// A transfer-setup request: a data-format identifier, a memory address and a memory
        /// size, plus the byte width each of the latter two is declared with on the wire.
        ///
        /// A width that would truncate its value is rejected rather than silently corrupting the
        /// address on the wire, and the crate-private `addressAndLengthFormatIdentifier` that
        /// packs the two widths never surfaces here.
        ///
        /// This doc comment is the published schema description for the request type, because its
        /// hand-written `PartialSchema` delegates here. It deliberately uses plain `///` lines:
        /// utoipa's derive reads only literal doc attributes, so a `#[doc = concat!(..)]` naming
        /// the specific service is dropped from the schema -- which is what left this description
        /// missing its verb.
        #[cfg(any(feature = "serde", feature = "utoipa"))]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
        // With `utoipa` but not `serde` these fields are never read: they exist to give the
        // schema its shape, which the derive reads at compile time rather than at run time.
        #[cfg_attr(not(feature = "serde"), allow(dead_code))]
        struct $repr {
            data_format_identifier: DataFormatIdentifier,
            memory_address: u64,
            memory_address_length: u8,
            memory_size: u32,
            memory_size_length: u8,
        }

        #[cfg(feature = "serde")]
        impl TryFrom<$repr> for $req {
            type Error = Error;

            fn try_from(repr: $repr) -> Result<Self, Error> {
                Self::new_with_widths(
                    repr.data_format_identifier,
                    repr.memory_address,
                    repr.memory_address_length,
                    repr.memory_size,
                    repr.memory_size_length,
                )
            }
        }

        #[cfg(feature = "serde")]
        impl From<$req> for $repr {
            fn from(request: $req) -> Self {
                Self {
                    data_format_identifier: request.data_format_identifier,
                    memory_address: request.memory_address,
                    memory_address_length: request
                        .address_and_length_format_identifier
                        .memory_address_length,
                    memory_size: request.memory_size,
                    memory_size_length: request
                        .address_and_length_format_identifier
                        .memory_size_length,
                }
            }
        }

        #[cfg(feature = "utoipa")]
        impl utoipa::PartialSchema for $req {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                <$repr as utoipa::PartialSchema>::schema()
            }
        }

        #[cfg(feature = "utoipa")]
        impl utoipa::ToSchema for $req {
            fn name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($req))
            }

            /// See the note on `CommunicationControlRequest::schemas`: a hand-written `ToSchema`
            /// must forward this or its `$ref`s dangle.
            fn schemas(
                schemas: &mut Vec<(
                    String,
                    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
                )>,
            ) {
                <$repr as utoipa::ToSchema>::schemas(schemas);
            }
        }

        impl Encode for $req {
            type Error = crate::Error;

            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                let mut written = write_all(
                    writer,
                    &[
                        self.data_format_identifier.into(),
                        self.address_and_length_format_identifier.into(),
                    ],
                )
                .map_err(Error::io)?;

                let addr_len = self
                    .address_and_length_format_identifier
                    .memory_address_length as usize;
                let size_len =
                    self.address_and_length_format_identifier.memory_size_length as usize;
                written += write_be_uint(writer, u128::from(self.memory_address), addr_len)?;
                written += write_be_uint(writer, u128::from(self.memory_size), size_len)?;

                Ok(written)
            }
        }

        impl<'a> Decode<'a> for $req {
            type Error = crate::Error;

            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                if buf.len() < 2 {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 2,
                        available: buf.len(),
                    }));
                }
                let data_format_identifier = DataFormatIdentifier::from(buf[0]);
                let memory_identifier = MemoryFormatIdentifier::try_from(buf[1])?;
                let addr_len = memory_identifier.memory_address_length as usize;
                let size_len = memory_identifier.memory_size_length as usize;
                let total = 2 + addr_len + size_len;
                if buf.len() < total {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: total,
                        available: buf.len(),
                    }));
                }

                let (memory_address, rest) = read_be_uint_into::<u64>(&buf[2..], addr_len)?;
                let (memory_size, _rest) = read_be_uint_into::<u32>(rest, size_len)?;

                Ok((
                    Self {
                        data_format_identifier,
                        address_and_length_format_identifier: memory_identifier,
                        memory_address,
                        memory_size,
                    },
                    &buf[total..],
                ))
            }
        }

        #[doc = $resp_doc]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub struct $resp<'d> {
            /// Maximum number of bytes per [`TransferDataRequest`](crate::TransferDataRequest).
            ///
            /// The on-wire `lengthFormatIdentifier` nibble is derived from this slice's length,
            /// so the declared length can never disagree with the bytes present. That nibble
            /// holds at most `0x0F`, which is why the slice is private and
            #[doc = concat!("[`", stringify!($resp), "::new`] is fallible.")]
            #[cfg_attr(feature = "serde", serde(borrow))]
            max_number_of_block_length: &'d [u8],
        }

        impl<'d> $resp<'d> {
            #[doc = concat!("Create a new `", stringify!($resp), "`. The `lengthFormatIdentifier`")]
            /// is derived from `max_number_of_block_length` during encoding.
            ///
            /// # Errors
            /// Returns [`Error::IncorrectMessageLengthOrInvalidFormat`] if the slice is longer
            /// than 15 bytes, which is the widest a single nibble can declare. The check used to
            /// live in `encode`, so a too-long slice was accepted here and failed later — the
            /// same construct-then-fail-to-encode asymmetry the transfer requests had.
            pub const fn new(max_number_of_block_length: &'d [u8]) -> Result<Self, Error> {
                if max_number_of_block_length.len() > MAX_BLOCK_LENGTH_BYTES {
                    return Err(Error::IncorrectMessageLengthOrInvalidFormat);
                }
                Ok(Self {
                    max_number_of_block_length,
                })
            }

            /// The maximum number of bytes the server will accept per `TransferData` request,
            /// as the raw big-endian bytes the server sent.
            #[must_use]
            pub const fn max_number_of_block_length(&self) -> &'d [u8] {
                self.max_number_of_block_length
            }
        }

        impl Encode for $resp<'_> {
            type Error = crate::Error;

            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                // `new` already bounded this to a nibble, so the cast cannot truncate.
                #[allow(clippy::cast_possible_truncation)]
                let nibble = self.max_number_of_block_length.len() as u8;
                let length_format_identifier = LengthFormatIdentifier {
                    max_number_of_block_length: nibble,
                };
                let mut written =
                    write_u8(writer, length_format_identifier.into()).map_err(Error::io)?;
                written += write_all(writer, self.max_number_of_block_length).map_err(Error::io)?;
                Ok(written)
            }
        }

        impl<'a> Decode<'a> for $resp<'a> {
            type Error = crate::Error;

            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 1,
                        available: buf.len(),
                    }));
                }
                let length_format_identifier = LengthFormatIdentifier::from(buf[0]);
                let len = length_format_identifier.max_number_of_block_length as usize;
                let total = 1 + len;
                if buf.len() < total {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: total,
                        available: buf.len(),
                    }));
                }
                Ok((
                    Self {
                        max_number_of_block_length: &buf[1..total],
                    },
                    &buf[total..],
                ))
            }
        }

        // Both services get the same coverage, generated alongside them so neither can drift.
        #[cfg(test)]
        mod $test_mod {
            use super::*;
            use crate::{Decode, Encode, test_util::assert_encode_size_agrees};

            #[test]
            fn simple_request() {
                let bytes: [u8; 7] = [
                    0x00, // No compression or encryption
                    0x14, // 1 byte for memory size, 4 bytes for memory address
                    0xF0, 0xFF, 0xFF, 0x67, // memory address
                    0x0A,
                ];
                let (req, _) = <$req as Decode>::decode(&bytes).unwrap();

                assert_eq!(u8::from(req.data_format_identifier), 0);
                assert_eq!(u8::from(req.address_and_length_format_identifier), 0x14);
                assert_eq!(
                    req.address_and_length_format_identifier.memory_size_length,
                    1
                );
                assert_eq!(
                    req.address_and_length_format_identifier
                        .memory_address_length,
                    4
                );

                assert_eq!(req.memory_address(), 0xF0FF_FF67);
                assert_eq!(req.memory_size(), 0x0A);
            }

            #[test]
            fn bad_request() {
                let bytes: [u8; 3] = [
                    0x00, // No compression or encryption
                    0x11, // 1 byte for memory size, 1 byte for memory address
                    0x67,
                ];
                let result = <$req as Decode>::decode(&bytes);
                assert!(result.is_err());
            }

            #[test]
            fn zero_address_and_size_clamp_to_one_byte() {
                // A 0 address/size must still produce a valid (>=1 byte) length nibble,
                // otherwise the encoded frame cannot be decoded back.
                let req = $req::new(0x00.into(), 0, 0).unwrap();
                assert_eq!(
                    req.address_and_length_format_identifier
                        .memory_address_length,
                    1
                );
                assert_eq!(
                    req.address_and_length_format_identifier.memory_size_length,
                    1
                );

                let mut buf = [0u8; 8];
                let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
                let (decoded, _) = <$req as Decode>::decode(&buf[..written]).unwrap();
                assert_eq!(decoded.memory_address(), 0);
                assert_eq!(decoded.memory_size(), 0);
            }

            #[test]
            fn check_message_size() {
                let req = $req::new(0x00.into(), 0xF0_FF_FF_67, 0x0A).unwrap();
                let mut buf = [0u8; 16];
                let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();

                assert_eq!(written, req.encoded_size().unwrap());
                assert_encode_size_agrees(&req);
            }

            #[test]
            fn new_derives_the_alfid_nibbles_in_wire_order() {
                // The two nibbles are asymmetric here on purpose: the high nibble is the
                // memorySize width and the low nibble the memoryAddress width, so transposing
                // them changes these bytes. A symmetric case (or a length-only assertion)
                // cannot tell the two apart, and a swap silently truncates the address.
                for (address, size, want) in [
                    // 4-byte address, 1-byte size -> ALFID 0x14
                    (
                        0xF0FF_FF67u64,
                        0x0Au32,
                        [0x00, 0x14, 0xF0, 0xFF, 0xFF, 0x67, 0x0A].as_slice(),
                    ),
                    // 1-byte address, 2-byte size -> ALFID 0x21
                    (0xBE, 0x0100, [0x00, 0x21, 0xBE, 0x01, 0x00].as_slice()),
                ] {
                    let req = $req::new(DataFormatIdentifier::NONE, address, size).unwrap();
                    let mut buf = [0u8; 16];
                    let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
                    assert_eq!(
                        &buf[..written],
                        want,
                        "wrong wire bytes for address {address:#X} size {size:#X}"
                    );

                    // ...and the frame the crate emits must be one the crate can read back.
                    let decoded = <$req as Decode>::decode_exact(&buf[..written]).unwrap();
                    assert_eq!(decoded.memory_address(), address);
                    assert_eq!(decoded.memory_size(), size);
                }
            }

            #[test]
            fn explicit_widths_can_reproduce_the_spec_example_alfid() {
                // ISO 14229-1:2020 Table 441 makes the addressAndLengthFormatIdentifier a
                // client choice, not a function of the values. Table 462's own example declares
                // 3 bytes of memorySize for the value 0x00FFFF, which needs only 2 -- so `new`,
                // which always derives minimal widths, cannot produce that frame. Real
                // bootloaders commonly mandate a fixed ALFID (often 0x44) and answer
                // requestOutOfRange otherwise.
                let req = $req::new_with_widths(
                    DataFormatIdentifier::from(0x11),
                    0x0060_2000,
                    3,
                    0x0000_FFFF,
                    3,
                )
                .unwrap();
                let mut buf = [0u8; 16];
                let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
                assert_eq!(
                    &buf[..written],
                    &[0x11, 0x33, 0x60, 0x20, 0x00, 0x00, 0xFF, 0xFF],
                );

                let decoded = <$req as Decode>::decode_exact(&buf[..written]).unwrap();
                assert_eq!(
                    decoded, req,
                    "a wider-than-minimal width must survive decode"
                );
            }

            #[test]
            fn explicit_widths_reject_a_value_that_does_not_fit() {
                // A declared width narrower than the value would silently truncate on the wire.
                assert!(
                    $req::new_with_widths(DataFormatIdentifier::NONE, 0x1_0000, 2, 1, 1).is_err()
                );
                assert!(
                    $req::new_with_widths(DataFormatIdentifier::NONE, 1, 1, 0x1_0000, 2).is_err()
                );
                // ...and the widths themselves must be ones Table H.1 permits.
                assert!($req::new_with_widths(DataFormatIdentifier::NONE, 1, 0, 1, 1).is_err());
                assert!($req::new_with_widths(DataFormatIdentifier::NONE, 1, 6, 1, 1).is_err());
                assert!($req::new_with_widths(DataFormatIdentifier::NONE, 1, 1, 1, 5).is_err());
            }

            #[test]
            fn the_widest_legal_address_and_size_round_trip() {
                // Annex H Table H.1 permits a 4-byte memorySize and a 5-byte memoryAddress
                // (ALFID 0x45), which `new` derives for these values. The decoder used to
                // reject both widths, so the crate could not read back its own output for any
                // transfer above 16 MB or to an address above 4 GB.
                let req =
                    $req::new(DataFormatIdentifier::NONE, 0xFF_FFFF_FFFF, 0xFFFF_FFFF).unwrap();
                let mut buf = [0u8; 16];
                let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
                assert_eq!(
                    &buf[..written],
                    &[
                        0x00, 0x45, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
                    ],
                );

                let decoded = <$req as Decode>::decode_exact(&buf[..written]).unwrap();
                assert_eq!(decoded.memory_address(), 0xFF_FFFF_FFFF);
                assert_eq!(decoded.memory_size(), 0xFFFF_FFFF);
            }

            #[test]
            fn address_beyond_five_bytes_is_rejected() {
                assert!(matches!(
                    $req::new(0x00.into(), 0x1_00_0000_0000, 0x0A),
                    Err(Error::InvalidMemoryAddress(_))
                ));
            }

            #[test]
            fn response_encode_size_agrees() {
                let block = [0x10u8, 0x00, 0x00];
                let resp = $resp::new(&block).unwrap();
                assert_encode_size_agrees(&resp);
            }

            #[test]
            fn a_reserved_length_nibble_is_zeroed_on_re_encode() {
                // ISO 14229-1:2020 Tables 443 and 448 both give the lengthFormatIdentifier the
                // range 0x00 to 0xF0 and state that bits 3 to 0 are "reserved by document, to be
                // set to '0'" -- "the lower nibble shall be set to '0'". So `74 25 08 00` is a
                // malformed byte, not a value to preserve: the block length is still read from
                // the high nibble, but re-encoding emits the conformant `74 20 08 00`.
                let resp = <$resp as Decode>::decode_exact(&[0x25, 0x08, 0x00]).unwrap();
                assert_eq!(resp.max_number_of_block_length(), &[0x08, 0x00]);

                let mut buf = [0u8; 8];
                let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
                assert_eq!(&buf[..written], &[0x20, 0x08, 0x00]);

                // And a response the crate builds itself encodes the same way.
                let fresh = $resp::new(&[0x08, 0x00]).unwrap();
                let written = Encode::encode(&fresh, &mut buf.as_mut_slice()).unwrap();
                assert_eq!(&buf[..written], &[0x20, 0x08, 0x00]);
            }

            #[test]
            fn a_block_length_wider_than_a_nibble_is_rejected_at_construction() {
                assert!($resp::new(&[0u8; 16]).is_err());
                assert!($resp::new(&[0u8; 15]).is_ok());
            }

            #[test]
            fn response_round_trips() {
                let block = [0x02u8, 0x00];
                let resp = $resp::new(&block).unwrap();
                let mut buf = [0u8; 8];
                let n = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
                assert_eq!(&buf[..n], &[0x20, 0x02, 0x00]);
                let (decoded, rest) = <$resp as Decode>::decode(&buf[..n]).unwrap();
                assert!(rest.is_empty());
                assert_eq!(decoded.max_number_of_block_length(), &block);
            }

            #[test]
            fn data_format_identifier_is_readable_off_a_decoded_request() {
                // A server has to act on the compression/encryption methods it was asked for.
                // All fields are private (the width nibbles must stay in sync with the values),
                // so without the getter the DFI was write-only: constructible but unreadable
                // after a decode.
                // Wire: DFI=0x21 (compression 2, encryption 1), ALFID=0x12 (size 1, addr 2),
                // addr=0xBEEF, size=0x10
                let wire = [0x21, 0x12, 0xBE, 0xEF, 0x10];
                let req = <$req as Decode>::decode_exact(&wire).unwrap();
                let dfi = req.data_format_identifier();
                assert_eq!(dfi.compression_method(), 0x02);
                assert_eq!(dfi.encryption_method(), 0x01);
                assert_eq!(u8::from(dfi), 0x21);
                assert_eq!(req.memory_address(), 0xBEEF);
                assert_eq!(req.memory_size(), 0x10);
            }

            #[test]
            fn data_format_identifier_survives_construction() {
                // new(compression, encryption) — wire order.
                let dfi = DataFormatIdentifier::new(0x0A, 0x0B).unwrap();
                let req = $req::new(dfi, 0x1234, 0x10).unwrap();
                assert_eq!(req.data_format_identifier(), dfi);
                assert_eq!(req.data_format_identifier().compression_method(), 0x0A);
                assert_eq!(req.data_format_identifier().encryption_method(), 0x0B);
            }

            #[test]
            fn exposes_allowed_nack_codes() {
                assert!(!$req::allowed_nack_codes().is_empty());
                assert!(
                    $req::allowed_nack_codes()
                        .contains(&NegativeResponseCode::UploadDownloadNotAccepted)
                );
            }

            #[test]
            fn derive_contract() {
                use crate::test_util::assert_impl_eq;
                assert_impl_eq::<$req>();
                assert_impl_eq::<$resp<'static>>();
                #[cfg(feature = "serde")]
                {
                    use crate::test_util::assert_impl_serde;
                    assert_impl_serde::<$req>();
                    assert_impl_serde::<$resp<'static>>();
                }
            }
        }
    };
}

upload_download_service! {
    request: RequestDownloadRequest,
    request_serde_repr: RequestDownloadRepr,
    request_serde_repr_name: "RequestDownloadRepr",
    response: RequestDownloadResponse,
    nrcs: REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES,
    request_doc: "A request to the server for it to download data from the client.\n\nA positive response ([`RequestDownloadResponse`]) is sent once the server has taken all necessary actions and is ready to receive the data.",
    response_doc: "Zero-alloc positive response to a [`RequestDownloadRequest`], indicating the server is ready to receive data. Borrows from the caller.",
    verb: "downloaded",
    tests: request_download_tests,
}

upload_download_service! {
    request: RequestUploadRequest,
    request_serde_repr: RequestUploadRepr,
    request_serde_repr_name: "RequestUploadRepr",
    response: RequestUploadResponse,
    nrcs: REQUEST_UPLOAD_NEGATIVE_RESPONSE_CODES,
    request_doc: "A request to the server for it to upload data to the client.\n\nA positive response ([`RequestUploadResponse`]) is sent once the server is ready to transmit; the client then drives the transfer with [`TransferDataRequest`](crate::TransferDataRequest), reading the data out of each positive response, and finishes with [`RequestTransferExitRequest`](crate::RequestTransferExitRequest).",
    response_doc: "Zero-alloc positive response to a [`RequestUploadRequest`], indicating the server is ready to transmit data. Borrows from the caller.",
    verb: "uploaded",
    tests: request_upload_tests,
}

#[cfg(test)]
mod shared_layout_tests {
    use super::*;
    use crate::Decode;

    #[test]
    fn download_and_upload_share_one_wire_layout() {
        // The two services are byte-identical apart from the service identifier, which is
        // added by the `Request`/`Response` frame layer, not by these payload codecs. This
        // pins that equivalence so the macro cannot silently drift for one of them.
        let wire = [0x00, 0x14, 0xF0, 0xFF, 0xFF, 0x67, 0x0A];
        let down = <RequestDownloadRequest as Decode>::decode_exact(&wire).unwrap();
        let up = <RequestUploadRequest as Decode>::decode_exact(&wire).unwrap();
        assert_eq!(down.memory_address(), up.memory_address());
        assert_eq!(down.memory_size(), up.memory_size());
        assert_eq!(down.data_format_identifier(), up.data_format_identifier());
        assert_eq!(
            down.encoded_size().unwrap(),
            up.encoded_size().unwrap(),
            "payload widths diverged"
        );
    }
}
