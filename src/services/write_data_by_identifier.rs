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
    /// Note: The first two bytes of the payload must be the data identifier immediately followed by the corresponding
    /// payload for that identifier.
    pub payload: Payload,
}

impl<Payload: IterableWireFormat> WriteDataByIdentifierRequest<Payload> {
    pub(crate) fn new(payload: Payload) -> Self {
        Self { payload }
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<Payload: IterableWireFormat> SingleValueWireFormat for WriteDataByIdentifierRequest<Payload> {}

impl<Payload: IterableWireFormat> WireFormat for WriteDataByIdentifierRequest<Payload> {
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let payload = Payload::option_from_reader(reader)?.unwrap();
        Ok(Some(Self { payload }))
    }

    fn required_size(&self) -> usize {
        // MJB TODO
        2 + self.data.len()
    }


    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.payload.to_writer(writer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::iris_generic_diagnostics;

    fn deserialize_write_data_by_identifier() {}

    fn serialize_write_data_by_identifier() {}
}
