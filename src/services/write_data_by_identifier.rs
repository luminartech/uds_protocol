//! `WriteDataByIdentifier` (0x2E) service implementation
use crate::{Decode, Encode, Error, NegativeResponseCode};

const WRITE_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// Zero-alloc TX request to write data by identifier. Borrows the raw payload from the caller.
///
/// The payload is the DID (2 bytes, big-endian) followed by the data record, exactly as
/// it appears on the wire after the service byte.
///
/// See ISO-14229-1:2020, Section 11.7.2.1
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriteDataByIdentifierRequestTx<'d> {
    /// The raw payload bytes: DID followed by the data record.
    pub payload: &'d [u8],
}

impl<'d> WriteDataByIdentifierRequestTx<'d> {
    /// Create a new write-by-identifier request from raw payload bytes.
    #[must_use]
    pub const fn new(payload: &'d [u8]) -> Self {
        Self { payload }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for WriteDataByIdentifierRequestTx<'_> {
    fn encoded_size(&self) -> usize {
        self.payload.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(self.payload).map_err(Error::io)?;
        Ok(self.payload.len())
    }
}

/// Positive response to `WriteDataByIdentifier`: echoes the DID that was written.
///
/// See ISO-14229-1:2020, Section 11.7.3.1
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
            return Err(Error::InsufficientData(2));
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
    fn test_write_request_encode() {
        // DID 0xF186 + one data byte 0x01
        let payload = [0xF1, 0x86, 0x01];
        let request = WriteDataByIdentifierRequestTx::new(&payload);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&request, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 3);
        assert_eq!(&buf[..3], &[0xF1, 0x86, 0x01]);
        assert_encode_size_agrees(&request);
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
        assert!(matches!(err, Err(Error::InsufficientData(2))));
    }
}
