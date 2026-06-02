//! `ReadDataByIdentifier` (0x22) service implementation
use crate::{Encode, Error, NegativeResponseCode};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// Zero-alloc TX request to read data by identifier. Borrows the DID list from the caller.
///
/// A Data Identifier is a 16-bit value, so the list is held as `&[u16]`; each DID is
/// written big-endian on the wire.
///
/// See ISO-14229-1:2020, Table 11.2.1 for format information
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadDataByIdentifierRequestTx<'d> {
    /// The list of Data Identifiers to read.
    pub dids: &'d [u16],
}

impl<'d> ReadDataByIdentifierRequestTx<'d> {
    /// Create a new request from a slice of data identifiers.
    #[must_use]
    pub const fn new(dids: &'d [u16]) -> Self {
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ReadDataByIdentifierRequestTx<'_> {
    fn encoded_size(&self) -> usize {
        self.dids.len() * 2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        for did in self.dids {
            writer.write_all(&did.to_be_bytes()).map_err(Error::io)?;
        }
        Ok(self.encoded_size())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_read_did_request_tx() {
        let ids = [0xF180u16, 0xF186u16];
        let req = ReadDataByIdentifierRequestTx::new(&ids);
        let mut buf = [0u8; 16];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 4); // 2 DIDs * 2 bytes each
        assert_eq!(&buf[..4], &[0xF1, 0x80, 0xF1, 0x86]);
        assert_encode_size_agrees(&req);
    }
}
