//! `ReadDataByIdentifier` (0x22) service implementation
use crate::{
    Encode, Error, Identifier, NegativeResponseCode,
};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// See ISO-14229-1:2020, Table 11.2.1 for format information
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ReadDataByIdentifierRequest<DataIdentifier> {
    /// The list of Data Identifiers to read.
    pub dids: Vec<DataIdentifier>,
}

impl<DataIdentifier: Identifier> ReadDataByIdentifierRequest<DataIdentifier> {
    /// Create a new request from a sequence of data identifiers
    pub(crate) fn new<I>(dids: I) -> Self
    where
        I: IntoIterator<Item = DataIdentifier>,
    {
        let dids = dids.into_iter().collect();
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<DataIdentifier: Identifier> Encode for ReadDataByIdentifierRequest<DataIdentifier> {
    fn encoded_size(&self) -> usize {
        self.dids.len() * 2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        for did in &self.dids {
            Encode::encode(did, writer)?;
        }
        Ok(self.encoded_size())
    }
}

/// See ISO-14229-1:2020, Table 11.2.3 for format information
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct ReadDataByIdentifierResponse<UserPayload> {
    /// The decoded payload entries returned by the server.
    pub data: Vec<UserPayload>,
}

impl<UserPayload> ReadDataByIdentifierResponse<UserPayload> {
    pub(crate) fn new<I>(data: I) -> Self
    where
        I: IntoIterator<Item = UserPayload>,
    {
        let data = data.into_iter().collect();
        Self { data }
    }
}

impl<UserPayload: Encode> Encode for ReadDataByIdentifierResponse<UserPayload> {
    fn encoded_size(&self) -> usize {
        self.data.iter().map(Encode::encoded_size).sum()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let mut total = 0;
        for item in &self.data {
            total += Encode::encode(item, writer)?;
        }
        Ok(total)
    }
}

// ---------------------------------------------------------------------------
// no_std TX type (borrow from caller)
// ---------------------------------------------------------------------------

/// Zero-alloc TX request to read data by identifier. Borrows DID list from caller.
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

impl<UserPayload: core::fmt::Debug> core::fmt::Debug
    for ReadDataByIdentifierResponse<UserPayload>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReadDataByIdentifierResponse\n{:?}", self.data)
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

    #[test]
    fn encode_read_did_request_alloc() {
        let ids = vec![
            ProtocolIdentifier::new(UDSIdentifier::BootSoftwareIdentification),
            ProtocolIdentifier::new(UDSIdentifier::ActiveDiagnosticSession),
        ];
        let req = ReadDataByIdentifierRequest::new(ids);
        let mut buf = [0u8; 16];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 4);
    }
}
