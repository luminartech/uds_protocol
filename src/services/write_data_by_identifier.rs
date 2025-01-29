use crate::{Error, IterableWireFormat, NegativeResponseCode, SingleValueWireFormat, WireFormat};
use serde::{Deserialize, Serialize};

const WRITE_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// See ISO-14229-1:2020, Section 11.7.2.1
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    /// Create an WriteDataByIdentifierRequest from a stream of bytes, i.e. deserialization.
    ///
    /// Note: The first two bytes in the reader must represent the data identifier, immediately followed by the
    /// corresponding payload for that identifier.
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let payload = Payload::option_from_reader(reader)?.unwrap();
        Ok(Some(Self { payload }))
    }

    fn required_size(&self) -> usize {
        // MJB TODO
        2 + self.data.len()
    }

    /// Serialize an WriteDataByIdentifierRequest instance.
    ///
    /// Note: The first two bytes of the writer will be the data identifier, immediately followed by the corresponding
    /// payload for that identifier.
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Payload must implement the extra bytes, because option_from_reader needs to know how to interpret payload message
        self.payload.to_writer(writer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use byteorder::WriteBytesExt;
    use std::io::Cursor;

    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
    pub enum TestIdentifier {
        Abracadabra = 0xBEEF,
    }

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

            writer.write(&id_bytes.to_be_bytes())?;

            match self {
                TestPayload::Abracadabra(value) => {
                    writer.write_u8(*value)?;
                    Ok(3)
                }
            }
        }
    }

    impl IterableWireFormat for TestPayload {}

    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_write_data_by_identifier_request() {
        let request = WriteDataByIdentifierRequest::new(TestPayload::Abracadabra(42));

        let mut bytes = Vec::new();
        let len = request.to_writer(&mut bytes).unwrap();
        assert_eq!(len, 3);
        assert_eq!(bytes.len(), len);

        let mut cursor = Cursor::new(bytes);
        let request2 = WriteDataByIdentifierRequest::<TestPayload>::option_from_reader(&mut cursor)
            .unwrap()
            .unwrap();
        assert_eq!(request, request2);
    }
}
