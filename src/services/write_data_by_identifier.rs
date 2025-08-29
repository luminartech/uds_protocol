use crate::{
    Error, Identifier, IterableWireFormat, NegativeResponseCode, SingleValueWireFormat, WireFormat,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const WRITE_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// See ISO-14229-1:2020, Section 11.7.2.1
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[non_exhaustive]
pub struct WriteDataByIdentifierRequest<Payload> {
    pub payload: Payload,
}

impl<Payload: IterableWireFormat> WriteDataByIdentifierRequest<Payload> {
    pub fn new(payload: Payload) -> Self {
        Self { payload }
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<Payload: IterableWireFormat> SingleValueWireFormat for WriteDataByIdentifierRequest<Payload> {}

impl<Payload: IterableWireFormat> WireFormat for WriteDataByIdentifierRequest<Payload> {
    /// Create a WriteDataByIdentifierRequest from a stream of bytes, i.e. deserialization.
    ///
    /// Note: The first two bytes in the reader must represent the data identifier, immediately followed by the
    /// corresponding payload for that identifier.
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let payload = Payload::option_from_reader(reader)?.unwrap();
        Ok(Some(Self { payload }))
    }

    fn required_size(&self) -> usize {
        self.payload.required_size()
    }

    /// Serialize a WriteDataByIdentifierRequest instance.
    ///
    /// Note: The first two bytes of the writer will be the data identifier, immediately followed by the corresponding
    /// payload for that identifier.
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Payload must implement the extra bytes, because option_from_reader needs to know how to interpret payload message
        self.payload.to_writer(writer)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

/// See ISO-14229-1:2020, Section 11.7.3.1
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[non_exhaustive]
pub struct WriteDataByIdentifierResponse<DataIdentifier> {
    pub identifier: DataIdentifier,
}

impl<DataIdentifier: Identifier> WriteDataByIdentifierResponse<DataIdentifier> {
    pub fn new(identifier: DataIdentifier) -> Self {
        Self { identifier }
    }
}

impl<DataIdentifier: Identifier> SingleValueWireFormat
    for WriteDataByIdentifierResponse<DataIdentifier>
{
}

impl<DataIdentifier: Identifier> WireFormat for WriteDataByIdentifierResponse<DataIdentifier> {
    /// Create a WriteDataByIdentifierResponse from a stream of bytes, i.e. deserialization.
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let identifier = DataIdentifier::option_from_reader(reader)?.unwrap();
        Ok(Some(Self::new(identifier)))
    }

    fn required_size(&self) -> usize {
        self.identifier.required_size()
    }

    /// Serialize a WriteDataByIdentifierResponse instance.
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Payload must implement the extra bytes, because option_from_reader needs to know how to interpret payload message
        self.identifier.to_writer(writer)
    }
}
///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use byteorder::WriteBytesExt;

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, Identifier)]
    pub enum TestIdentifier {
        Abracadabra = 0xBEEF,
    }
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

    impl PartialEq<u16> for TestIdentifier {
        fn eq(&self, other: &u16) -> bool {
            match self {
                TestIdentifier::Abracadabra => *other == 0xBEEF,
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
    enum TestPayload {
        Abracadabra(u8),
    }

    impl WireFormat for TestPayload {
        fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;

            let value = u16::from_be_bytes(buf);

            if value == TestIdentifier::Abracadabra as u16 {
                let mut byte = [0u8; 1];
                reader.read_exact(&mut byte)?;
                Ok(Some(TestPayload::Abracadabra(byte[0])))
            } else {
                Err(Error::NoDataAvailable)
            }
        }

        fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
            let id_bytes: u16 = match self {
                TestPayload::Abracadabra(_) => 0xBEEF,
            };

            writer.write_all(&id_bytes.to_be_bytes())?;

            match self {
                TestPayload::Abracadabra(value) => {
                    writer.write_u8(*value)?;
                    Ok(self.required_size())
                }
            }
        }

        fn required_size(&self) -> usize {
            3
        }
    }

    impl IterableWireFormat for TestPayload {}

    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_write_request() {
        let request = WriteDataByIdentifierRequest::new(TestPayload::Abracadabra(42));

        let mut written_bytes = Vec::new();
        let written = request.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, request.required_size());
        assert_eq!(written, written_bytes.len());

        let request2 = WriteDataByIdentifierRequest::<TestPayload>::option_from_reader(
            &mut written_bytes.as_slice(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(request, request2);
    }

    #[test]
    fn test_write_response() {
        let response = WriteDataByIdentifierResponse::new(TestIdentifier::Abracadabra);

        let mut written_bytes = Vec::new();
        let written = response.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, response.required_size());
    }
}
