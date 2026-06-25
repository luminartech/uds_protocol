//! `RequestDownload` (0x34) service implementation

use crate::shared::{
    DataFormatIdentifier, LengthFormatIdentifier, MemoryFormatIdentifier, read_be_uint,
    write_be_uint,
};
use crate::{Decode, Encode, Error, NegativeResponseCode};

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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct RequestDownloadRequest {
    /// compression method (high nibble) and encrypting method (low nibble). 0x00 is no compression or encryption
    data_format_identifier: DataFormatIdentifier,
    /// 7-4: length (# of bytes) of `memory_size` param, 3-0: length (# of bytes) of `memory_address` param
    address_and_length_format_identifier: MemoryFormatIdentifier,
    /// Starting address of the server memory. Size is determined by `address_and_length_format_identifier`
    /// Has a variable number of bytes, max of 5
    pub memory_address: u64,
    /// Size of the data to be downloaded. Number of bytes sent is determined by `address_and_length_format_identifier`
    /// Used by the server to validate the data transferred by the [`TransferDataRequest`](crate::TransferDataRequest) service
    /// Has a variable number of bytes, max of 4
    pub memory_size: u32,
}

impl RequestDownloadRequest {
    /// Create a new `RequestDownloadRequest`
    ///
    /// # Errors
    /// Returns an error if `memory_address` exceeds 5 bytes (> `0xFF_FFFF_FFFF`).
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(
        data_format_identifier: DataFormatIdentifier,
        memory_address: u64,
        memory_size: u32,
    ) -> Result<Self, Error> {
        if memory_address > 0xFF_FFFF_FFFF {
            return Err(Error::InvalidMemoryAddress(memory_address));
        }
        // A length of 0 produces an invalid `MemoryFormatIdentifier` (the nibbles
        // must be >=1 per ISO-14229), so clamp to at least one byte even when the
        // address or size is 0.
        let memory_address_length =
            ((u64::BITS - memory_address.leading_zeros()).div_ceil(8) as u8).max(1);
        let memory_size_length =
            ((u32::BITS - memory_size.leading_zeros()).div_ceil(8) as u8).max(1);
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

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &REQUEST_DOWNLOAD_NEGATIVE_RESPONSE_CODES
    }
}
impl Encode for RequestDownloadRequest {
    fn encoded_size(&self) -> usize {
        2 + self.address_and_length_format_identifier.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[
                self.data_format_identifier.into(),
                self.address_and_length_format_identifier.into(),
            ])
            .map_err(Error::io)?;

        let addr_len = self
            .address_and_length_format_identifier
            .memory_address_length as usize;
        let size_len = self.address_and_length_format_identifier.memory_size_length as usize;
        write_be_uint(u128::from(self.memory_address), addr_len, writer)?;
        write_be_uint(u128::from(self.memory_size), size_len, writer)?;

        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RequestDownloadRequest {
    #[allow(clippy::cast_possible_truncation)]
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let data_format_identifier = DataFormatIdentifier::from(buf[0]);
        let memory_identifier = MemoryFormatIdentifier::try_from(buf[1])?;
        let addr_len = memory_identifier.memory_address_length as usize;
        let size_len = memory_identifier.memory_size_length as usize;
        let total = 2 + addr_len + size_len;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }

        let memory_address = read_be_uint(&buf[2..], addr_len)? as u64;
        let memory_size = read_be_uint(&buf[2 + addr_len..], size_len)? as u32;

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

/// Zero-alloc response for request download. Borrows from the caller.
///
/// Positive response to a [`RequestDownloadRequest`] indicating the server is ready to receive data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestDownloadResponse<'d> {
    /// Maximum number of bytes per [`TransferDataRequest`](crate::TransferDataRequest).
    ///
    /// The on-wire `lengthFormatIdentifier` nibble is derived from this slice's length
    /// at encode time, so the declared length can never disagree with the bytes present.
    pub max_number_of_block_length: &'d [u8],
}

impl<'d> RequestDownloadResponse<'d> {
    /// Create a new request download response. The `lengthFormatIdentifier` is derived
    /// from `max_number_of_block_length` during encoding.
    #[must_use]
    pub const fn new(max_number_of_block_length: &'d [u8]) -> Self {
        Self {
            max_number_of_block_length,
        }
    }
}

impl Encode for RequestDownloadResponse<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.max_number_of_block_length.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        // The block-length field width is carried in a single nibble, so the slice
        // can be at most 0x0F bytes long.
        let nibble = u8::try_from(self.max_number_of_block_length.len())
            .ok()
            .filter(|n| *n <= 0x0F)
            .ok_or(Error::IncorrectMessageLengthOrInvalidFormat)?;
        let length_format_identifier = LengthFormatIdentifier {
            max_number_of_block_length: nibble,
        };
        writer
            .write_all(&[length_format_identifier.into()])
            .map_err(Error::io)?;
        writer
            .write_all(self.max_number_of_block_length)
            .map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RequestDownloadResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let length_format_identifier = LengthFormatIdentifier::from(buf[0]);
        let len = length_format_identifier.max_number_of_block_length as usize;
        let total = 1 + len;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }
        Ok((
            Self {
                max_number_of_block_length: &buf[1..total],
            },
            &buf[total..],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[test]
    fn simple_request() {
        let bytes: [u8; 7] = [
            0x00, // No compression or encryption
            0x14, // 1 byte for memory size, 4 bytes for memory address
            0xF0, 0xFF, 0xFF, 0x67, // memory address
            0x0A,
        ];
        let (req, _) = <RequestDownloadRequest as Decode>::decode(&bytes).unwrap();

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

        assert_eq!(req.memory_address, 0xF0FF_FF67);
        assert_eq!(req.memory_size, 0x0A);
    }

    #[test]
    fn bad_request() {
        let bytes: [u8; 3] = [
            0x00, // No compression or encryption
            0x11, // 1 byte for memory size, 1 byte for memory address
            0x67,
        ];
        let result = <RequestDownloadRequest as Decode>::decode(&bytes);
        assert!(result.is_err());
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

    #[test]
    fn zero_address_and_size_clamp_to_one_byte() {
        // A 0 address/size must still produce a valid (>=1 byte) length nibble,
        // otherwise the encoded frame cannot be decoded back.
        let req = RequestDownloadRequest::new(0x00.into(), 0, 0).unwrap();
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
        let (decoded, _) = <RequestDownloadRequest as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded.memory_address, 0);
        assert_eq!(decoded.memory_size, 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn check_message_size() {
        let req = RequestDownloadRequest::new(0x00.into(), 0xF0_FF_FF_67, 0x0A).unwrap();

        let mut vec = vec![];
        Encode::encode(&req, &mut vec).unwrap();

        assert_eq!(vec.len(), req.encoded_size());
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn response_encode_size_agrees() {
        let block = [0x10u8, 0x00, 0x00];
        let resp = RequestDownloadResponse::new(&block);
        assert_encode_size_agrees(&resp);
    }
}
