use crate::{
    Error, NegativeResponseCode, ResetType, SingleValueWireFormat, SuppressablePositiveResponse,
    WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

const ECU_RESET_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::SecurityAccessDenied,
];

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let reset_type = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        Ok(Some(Self { reset_type }))
    }

    fn required_size(&self) -> usize {
        1
    }

    /// Serialization function to write a [`EcuResetRequest`] to a `Writer`
    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.reset_type))?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl SingleValueWireFormat for EcuResetRequest {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct EcuResetResponse {
    pub reset_type: ResetType,
    /// Optional power down time in seconds for [ResetType::EnableRapidPowerShutDown]
    pub power_down_time: Option<u8>,
}

impl EcuResetResponse {
    /// Create a new '`EcuResetResponse`'
    ///
    /// `power_down_time` is only valid for [ResetType::EnableRapidPowerShutDown], will be set to None otherwise
    pub(crate) fn new(reset_type: ResetType, power_down_time: Option<u8>) -> Self {
        let power_down_time = if reset_type == ResetType::EnableRapidPowerShutDown {
            power_down_time
        } else {
            None
        };
        Self {
            reset_type,
            power_down_time,
        }
    }
}

impl WireFormat for EcuResetResponse {
    /// Deserialization function to read a [`EcuResetResponse`] from a `Reader`
    fn decode<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let reset_type = ResetType::try_from(reader.read_u8()?)?;
        let mut power_down_time = None;
        if reset_type == ResetType::EnableRapidPowerShutDown {
            power_down_time = Some(reader.read_u8()?);
        }
        Ok(Some(Self {
            reset_type,
            power_down_time,
        }))
    }

    fn required_size(&self) -> usize {
        if self.reset_type == ResetType::EnableRapidPowerShutDown {
            2
        } else {
            1
        }
    }

    /// Serialization function to write a [`EcuResetResponse`] to a `Writer`
    fn encode<T: Write>(&self, buffer: &mut T) -> Result<usize, Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        if self.reset_type == ResetType::EnableRapidPowerShutDown {
            if let Some(power_down_time) = self.power_down_time {
                buffer.write_u8(power_down_time)?;
            } else {
                return Err(Error::SerializationError(
                    "ECUReset: Power down time must be set for EnableRapidPowerShutDown"
                        .to_string(),
                ));
            }
        }
        Ok(self.required_size())
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
        let written = req.encode(&mut buffer).unwrap();
        let result = EcuResetRequest::decode_single_value(&mut bytes.as_slice()).unwrap();
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
        let bytes: [u8; 2] = [0x04, 0x20];
        let resp = EcuResetResponse::new(ResetType::EnableRapidPowerShutDown, Some(0x20));
        let mut buffer = Vec::new();
        let written = resp.encode(&mut buffer).unwrap();
        let result = EcuResetResponse::decode_single_value(&mut bytes.as_slice()).unwrap();
        assert_eq!(result, resp);

        assert_eq!(written, 2);
        assert_eq!(written, resp.required_size());
    }

    #[test]
    // Test that power down time is ignored for other reset types
    fn ecu_reset_response_no_power_down_time() {
        let bytes: [u8; 1] = [0x01];
        let resp = EcuResetResponse::new(ResetType::HardReset, Some(0x20));
        let mut buffer = Vec::new();
        let written = resp.encode(&mut buffer).unwrap();
        assert_eq!(written, 1);
        let result = EcuResetResponse::decode_single_value(&mut bytes.as_slice()).unwrap();
        assert_eq!(result, EcuResetResponse::new(ResetType::HardReset, None));
    }
}
