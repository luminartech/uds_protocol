//! `ReadDataByIdentifier` (0x22) service implementation
use crate::{Encode, Error, Identifier, NegativeResponseCode};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// Zero-alloc TX request to read data by identifier. Borrows DID list from caller.
///
/// See ISO-14229-1:2020, Table 11.2.1 for format information
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadDataByIdentifierRequestTx<'d, DataIdentifier> {
    /// The list of Data Identifiers to read.
    pub dids: &'d [DataIdentifier],
}

impl<'d, DataIdentifier: Identifier> ReadDataByIdentifierRequestTx<'d, DataIdentifier> {
    /// Create a new request from a slice of data identifiers.
    #[must_use]
    pub const fn new(dids: &'d [DataIdentifier]) -> Self {
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<DataIdentifier: Identifier> Encode for ReadDataByIdentifierRequestTx<'_, DataIdentifier> {
    fn encoded_size(&self) -> usize {
        self.dids.len() * 2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        for did in self.dids {
            Encode::encode(did, writer)?;
        }
        Ok(self.encoded_size())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ProtocolIdentifier, UDSIdentifier};

    #[test]
    fn encode_read_did_request_tx() {
        let ids = [
            ProtocolIdentifier::new(UDSIdentifier::BootSoftwareIdentification),
            ProtocolIdentifier::new(UDSIdentifier::ActiveDiagnosticSession),
        ];
        let req = ReadDataByIdentifierRequestTx::new(&ids);
        let mut buf = [0u8; 16];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 4); // 2 DIDs * 2 bytes each
    }
}
