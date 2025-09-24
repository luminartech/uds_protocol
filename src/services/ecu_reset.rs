use crate::{
    Error, NegativeResponseCode, ResetType, SingleValueWireFormat, SuppressablePositiveResponse,
    WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use utoipa::ToSchema;

const ECU_RESET_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::SecurityAccessDenied,
];

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToSchema)]
/// Request for the server to reset the ECU
pub struct EcuResetRequest {
    reset_type: SuppressablePositiveResponse<ResetType>,
}

impl EcuResetRequest {
    /// Create a new '`EcuResetRequest`'
    pub(crate) fn new(suppress_positive_response: bool, reset_type: ResetType) -> Self {
        Self {
            reset_type: SuppressablePositiveResponse::new(suppress_positive_response, reset_type),
        }
    }

    /// Getter for whether a positive response should be suppressed
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.reset_type.suppress_positive_response()
    }

    /// Getter for the requested [`ResetType`]
    #[must_use]
    pub fn reset_type(&self) -> ResetType {
        self.reset_type.value()
    }

    /// Get the allowed Nack codes for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &ECU_RESET_NEGATIVE_RESPONSE_CODES
    }
}

impl WireFormat for EcuResetRequest {
    /// Deserialization function to read a [`EcuResetRequest`] from a `Reader`
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let reset_type = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        Ok(Some(Self { reset_type }))
    }

    fn required_size(&self) -> usize {
        1
    }

    /// Serialization function to write a [`EcuResetRequest`] to a `Writer`
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.reset_type))?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl SingleValueWireFormat for EcuResetRequest {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToSchema)]
#[non_exhaustive]
pub struct EcuResetResponse {
    pub reset_type: ResetType,
    pub power_down_time: u8,
}

impl EcuResetResponse {
    /// Create a new '`EcuResetResponse`'
    pub(crate) fn new(reset_type: ResetType, power_down_time: u8) -> Self {
        Self {
            reset_type,
            power_down_time,
        }
    }
}

impl WireFormat for EcuResetResponse {
    /// Deserialization function to read a [`EcuResetResponse`] from a `Reader`
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let reset_type = ResetType::try_from(reader.read_u8()?)?;
        let power_down_time = reader.read_u8()?;
        Ok(Some(Self {
            reset_type,
            power_down_time,
        }))
    }

    fn required_size(&self) -> usize {
        2
    }

    /// Serialization function to write a [`EcuResetResponse`] to a `Writer`
    fn to_writer<T: Write>(&self, buffer: &mut T) -> Result<usize, Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        buffer.write_u8(self.power_down_time)?;
        Ok(2)
    }
}

impl SingleValueWireFormat for EcuResetResponse {}

#[cfg(test)]
mod request {
    use super::*;

    #[test]
    fn ecu_reset_request() {
        let bytes: [u8; 2] = [0x81, 0x00];
        let req = EcuResetRequest::new(true, ResetType::HardReset);
        let mut buffer = Vec::new();
        let written = req.to_writer(&mut buffer).unwrap();
        let result = EcuResetRequest::from_reader(&mut bytes.as_slice()).unwrap();
        assert_eq!(result, req);

        assert_eq!(written, 1);
        assert_eq!(written, req.required_size());
    }
}

#[cfg(test)]
mod response {
    use super::*;

    #[test]
    fn ecu_reset_response() {
        let bytes: [u8; 2] = [0x01, 0x20];
        let resp = EcuResetResponse::new(ResetType::HardReset, 0x20);
        let mut buffer = Vec::new();
        let written = resp.to_writer(&mut buffer).unwrap();
        let result = EcuResetResponse::from_reader(&mut bytes.as_slice()).unwrap();
        assert_eq!(result, resp);

        assert_eq!(written, 2);
        assert_eq!(written, resp.required_size());
    }
}
