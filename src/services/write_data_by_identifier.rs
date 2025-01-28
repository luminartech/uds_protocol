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
pub struct WriteDataByIdentifierRequest<Identifier, Payload> {
    // pub identifier: Identifier,
    pub payload: Payload,
}

impl<Identifier: IterableWireFormat, Payload: IterableWireFormat>
    WriteDataByIdentifierRequest<Identifier, Payload>
{
    pub fn new(identifier: Identifier, payload: Payload) -> Self {
        Self {
            identifier,
            payload,
        }
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<Identifier: IterableWireFormat, Payload: IterableWireFormat> SingleValueWireFormat
    for WriteDataByIdentifierRequest<Identifier, Payload>
{
}

impl<Identifier: IterableWireFormat, Payload: IterableWireFormat> WireFormat
    for WriteDataByIdentifierRequest<Identifier, Payload>
{
    /// Create an WriteDataByIdentifierRequest from a stream of bytes, i.e. deserialization
    /// Note: The first two bytes in the reader must represent the data identifier, immediately followed by the
    /// corresponding payload for that identifier.    
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        // MJB: What to do? Can't get the identifier separate because option_from_reader needs to know
        // how to interpret the bytes. If we do that though, we lose the identifier.

        let identifier = Identifier::option_from_reader(reader)?.unwrap();
        let payload = Payload::option_from_reader(reader)?.unwrap();
        // Basically want something like this:

        // Force the payload to convert to Identifier?
        //let id = Payload::from(payload);
        // Force it to be constructible with new?
        //let payload = Payload::new(identifier, reader)?;

        Ok(Some(Self::new(identifier, payload)))
    }

    fn required_size(&self) -> usize {
        // MJB TODO
        2 + self.data.len()
    }

    /// Serialize an WriteDataByIdentifierRequest instance.
    /// Note: The first two bytes of the writer will be the data identifier, immediately followed by the corresponding
    /// payload for that identifier.
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // It would be nice if we could do this, but we can't
        //let mut bytes_written = self.identifier.to_writer(writer)?;
        // let bytes_written = self.payload.to_writer(writer)?;
        // Ok(bytes_written)

        // Payload must implement the extra bytes, because option_from_reader needs to know how to interpret payload message
        let bytes_written = self.payload.to_writer(writer)?;
        Ok(bytes_written)
    }
}
