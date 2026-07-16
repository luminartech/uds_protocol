//! `WriteDataByIdentifier` (0x2E) service implementation
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};

const WRITE_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// Zero-alloc request to write data by identifier. Borrows the data record from the caller.
///
/// The `identifier` is the 2-byte big-endian Data Identifier (DID); `data` is the opaque
/// data record that follows it on the wire.
///
/// See ISO-14229-1:2020, Section 11.7.2.1
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct WriteDataByIdentifierRequest<'d> {
    /// The 2-byte big-endian Data Identifier (DID) being written. Any `u16` is valid;
    /// the big-endian encoding is handled on the wire.
    pub identifier: u16,
    /// The opaque data record written after the identifier.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub data: &'d [u8],
}

impl<'d> WriteDataByIdentifierRequest<'d> {
    /// Create a request to write `data` to the given Data Identifier.
    #[must_use]
    pub const fn new(identifier: u16, data: &'d [u8]) -> Self {
        Self { identifier, data }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for WriteDataByIdentifierRequest<'_> {
    fn encoded_size(&self) -> usize {
        2 + self.data.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&self.identifier.to_be_bytes())
            .map_err(Error::io)?;
        writer.write_all(self.data).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for WriteDataByIdentifierRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(Incomplete {
                needed: 2,
                available: buf.len(),
            }));
        }
        let identifier = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((
            Self {
                identifier,
                data: &buf[2..],
            },
            &[],
        ))
    }
}

/// Positive response to `WriteDataByIdentifier`: echoes the DID that was written.
///
/// See ISO-14229-1:2020, Section 11.7.3.1
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct WriteDataByIdentifierResponse {
    /// The DID that was written to.
    pub identifier: u16,
}

impl WriteDataByIdentifierResponse {
    /// Create a new response echoing the identifier that was written.
    #[must_use]
    pub const fn new(identifier: u16) -> Self {
        Self { identifier }
    }
}

impl Encode for WriteDataByIdentifierResponse {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&self.identifier.to_be_bytes())
            .map_err(Error::io)?;
        Ok(2)
    }
}

impl<'a> Decode<'a> for WriteDataByIdentifierResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(Incomplete {
                needed: 2,
                available: buf.len(),
            }));
        }
        let identifier = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((Self { identifier }, &buf[2..]))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn derive_contract() {
        use crate::test_util::assert_impl_eq;
        assert_impl_eq::<WriteDataByIdentifierRequest<'static>>();
        assert_impl_eq::<WriteDataByIdentifierResponse>();
        #[cfg(feature = "serde")]
        {
            use crate::test_util::assert_impl_serde;
            assert_impl_serde::<WriteDataByIdentifierRequest<'_>>();
            assert_impl_serde::<WriteDataByIdentifierResponse>();
        }
    }

    #[test]
    fn test_write_response_encode() {
        let response = WriteDataByIdentifierResponse::new(0xBEEF);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&response, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 2);
        assert_eq!(buf[0], 0xBE);
        assert_eq!(buf[1], 0xEF);
        assert_encode_size_agrees(&response);
    }

    #[test]
    fn write_response_roundtrip() {
        let response = WriteDataByIdentifierResponse::new(0xF186);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&response, &mut buf.as_mut_slice()).unwrap();
        let (decoded, rest) =
            <WriteDataByIdentifierResponse as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, response);
        assert!(rest.is_empty());
    }

    #[test]
    fn write_response_decode_rejects_short_buffer() {
        let err = <WriteDataByIdentifierResponse as Decode>::decode(&[0x01]);
        assert!(matches!(err, Err(Error::InsufficientData(i)) if i.needed == 2 && i.available == 1));
    }

    #[test]
    fn wdbi_request_round_trips() {
        let req = WriteDataByIdentifierRequest::new(0xF190, &[0x01, 0x02, 0x03]);
        let mut buf = [0u8; 8];
        let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &[0xF1, 0x90, 0x01, 0x02, 0x03]);
        let (decoded, rest) = <WriteDataByIdentifierRequest as Decode>::decode(&buf[..n]).unwrap();
        assert!(rest.is_empty());
        assert_eq!(decoded.identifier, 0xF190);
        assert_eq!(decoded.data, &[0x01, 0x02, 0x03]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn wdbi_request_allows_empty_data() {
        let (decoded, _) = <WriteDataByIdentifierRequest as Decode>::decode(&[0xF1, 0x90]).unwrap();
        assert_eq!(decoded.identifier, 0xF190);
        assert!(decoded.data.is_empty());
    }

    #[test]
    fn wdbi_request_rejects_short_buffer() {
        assert!(matches!(
            <WriteDataByIdentifierRequest as Decode>::decode(&[0xF1]),
            Err(Error::InsufficientData(i)) if i.needed == 2 && i.available == 1
        ));
    }
}
