//! `TransferData` (0x36) service implementation

use crate::{Decode, Encode, Error, NegativeResponseCode};

const TRANSFER_DATA_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 6] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::RequestSequenceError,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::TransferDataSuspended,
    NegativeResponseCode::GeneralProgrammingFailure,
    NegativeResponseCode::WrongBlockSequenceCounter,
];

/// A request to the server to transfer data (either upload or download)
///
/// Step 1: The client sends a `RequestDownload` or `RequestUpload` message to the server
///     34 .. 11  .. 33   .. 60 20 00 .. 00 FF FF << -- Bytes sent by the client
///    RID .. DFI .. ALFID .. `MA_B`#   .. `UCMS_B`#
///
/// Step 1 Response: The server sends a [`RequestDownloadResponse`](crate::RequestDownloadResponse) or `RequestUploadResponse` message to the client
///
/// Step 2: The client shall send many [`TransferDataRequest`] messages written in blocks
///     to the server with a max number of bytes equal to `MNROB_B`# from the `RequestDownloadResponse` message
///    74  .. 20   .. 00 81
///   RSID .. LFID .. `MNROB_B`#
///
/// Step 2 Response: The server sends a [`TransferDataResponse`] message confirming the block sequence
///
/// Step 3: The client sends a [`crate::UdsServiceType::RequestTransferExit`] message to the server (SID 0x37)
///
/// Step 3 Response: The server sends a [`crate::UdsServiceType::RequestTransferExit`] response message to the client (RID 0x77)
///
/// Zero-alloc request to transfer data. Borrows from the caller.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct TransferDataRequest<'d> {
    /// Block sequence counter (wraps 0xFF → 0x00).
    pub block_sequence_counter: u8,
    /// The data to be transferred.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub data: &'d [u8],
}

impl<'d> TransferDataRequest<'d> {
    /// Create a new transfer data request.
    #[must_use]
    pub const fn new(block_sequence_counter: u8, data: &'d [u8]) -> Self {
        Self {
            block_sequence_counter,
            data,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &TRANSFER_DATA_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for TransferDataRequest<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.data.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[self.block_sequence_counter])
            .map_err(Error::io)?;
        writer.write_all(self.data).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for TransferDataRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((
            Self {
                block_sequence_counter: buf[0],
                data: &buf[1..],
            },
            &[],
        ))
    }
}

/// Zero-alloc response for transfer data. Borrows from the caller.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct TransferDataResponse<'d> {
    /// Echo of the block sequence counter.
    pub block_sequence_counter: u8,
    /// Response data (vendor-specific).
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub data: &'d [u8],
}

impl<'d> TransferDataResponse<'d> {
    /// Create a new transfer data response.
    #[must_use]
    pub const fn new(block_sequence_counter: u8, data: &'d [u8]) -> Self {
        Self {
            block_sequence_counter,
            data,
        }
    }
}

impl Encode for TransferDataResponse<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.data.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[self.block_sequence_counter])
            .map_err(Error::io)?;
        writer.write_all(self.data).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for TransferDataResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((
            Self {
                block_sequence_counter: buf[0],
                data: &buf[1..],
            },
            &[],
        ))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, NegativeResponseCode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[test]
    fn test_allowed_nack_codes() {
        let codes = TransferDataRequest::allowed_nack_codes();
        assert!(codes.contains(&NegativeResponseCode::WrongBlockSequenceCounter));
    }

    #[test]
    fn derive_contract() {
        use crate::test_util::assert_impl_eq;
        assert_impl_eq::<TransferDataRequest<'static>>();
        assert_impl_eq::<TransferDataResponse<'static>>();
        #[cfg(feature = "serde")]
        {
            use crate::test_util::assert_impl_serde;
            assert_impl_serde::<TransferDataRequest<'_>>();
            assert_impl_serde::<TransferDataResponse<'_>>();
        }
    }

    #[test]
    fn test_transfer_data_request() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let req = TransferDataRequest::new(0x01, &data);
        assert_eq!(1, req.block_sequence_counter);
        assert_eq!(req.data, &[0x01, 0x02, 0x03, 0x04]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn read_request() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let (req, _) = <TransferDataRequest as Decode>::decode(&bytes).unwrap();

        let mut written_bytes = Vec::new();
        let written = Encode::encode(&req, &mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, req.encoded_size());
        assert_encode_size_agrees(&req);
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn simple_response() {
        let bytes = [0x01, 0x02, 0x03, 0x04];
        let (resp, _) = <TransferDataResponse as Decode>::decode(&bytes).unwrap();

        let mut written_bytes = Vec::new();
        let written = Encode::encode(&resp, &mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, resp.encoded_size());
        assert_encode_size_agrees(&resp);
    }
}
