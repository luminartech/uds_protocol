//! `WriteDataByIdentifier` (0x2E) service implementation
use crate::{Encode, Error, Identifier, NegativeResponseCode};

const WRITE_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// See ISO-14229-1:2020, Section 11.7.2.1
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct WriteDataByIdentifierRequest<Payload> {
    /// The payload to write, which includes the DID and data.
    pub payload: Payload,
}

impl<Payload: Encode> WriteDataByIdentifierRequest<Payload> {
    /// Create a new write-by-identifier request.
    pub fn new(payload: Payload) -> Self {
        Self { payload }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<Payload: Encode> Encode for WriteDataByIdentifierRequest<Payload> {
    fn encoded_size(&self) -> usize {
        self.payload.encoded_size()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        self.payload.encode(writer)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

/// See ISO-14229-1:2020, Section 11.7.3.1
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct WriteDataByIdentifierResponse<DataIdentifier> {
    /// The DID that was written to.
    pub identifier: DataIdentifier,
}

impl<DataIdentifier: Identifier> WriteDataByIdentifierResponse<DataIdentifier> {
    /// Create a new response echoing the identifier that was written.
    pub fn new(identifier: DataIdentifier) -> Self {
        Self { identifier }
    }
}

impl<DataIdentifier: Identifier> Encode for WriteDataByIdentifierResponse<DataIdentifier> {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        Encode::encode(&self.identifier, writer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ProtocolPayloadTx, UDSIdentifier, impl_identifier};

    #[test]
    fn test_write_response_encode() {
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum TestIdentifier {
            Abracadabra = 0xBEEF,
        }
        impl_identifier!(TestIdentifier);
        impl From<u16> for TestIdentifier {
            fn from(value: u16) -> Self {
                match value {
                    0xBEEF => TestIdentifier::Abracadabra,
                    _ => panic!("Invalid test identifier: {value}"),
                }
            }
        }
        impl From<TestIdentifier> for u16 {
            fn from(value: TestIdentifier) -> Self {
                match value {
                    TestIdentifier::Abracadabra => 0xBEEF,
                }
            }
        }

        let response = WriteDataByIdentifierResponse::new(TestIdentifier::Abracadabra);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&response, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 2);
        assert_eq!(buf[0], 0xBE);
        assert_eq!(buf[1], 0xEF);
    }

    #[test]
    fn test_write_request_encode() {
        let payload = ProtocolPayloadTx::new(UDSIdentifier::ActiveDiagnosticSession, &[0x01]);
        let request = WriteDataByIdentifierRequest::new(payload);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&request, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 3);
    }
}
