use crate::{
    Error, NegativeResponseCode, ResetType, SingleValueWireFormat, SuppressablePositiveResponse,
    WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const ECU_RESET_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::SecurityAccessDenied,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Request for the server to reset the ECU
pub struct EcuResetRequest {
    reset_type: SuppressablePositiveResponse<ResetType>,
}

impl EcuResetRequest {
    /// Create a new 'EcuResetRequest'
    pub(crate) fn new(suppress_positive_response: bool, reset_type: ResetType) -> Self {
        Self {
            reset_type: SuppressablePositiveResponse::new(suppress_positive_response, reset_type),
        }
    }

    /// Getter for whether a positive response should be suppressed
    pub fn suppress_positive_response(&self) -> bool {
        self.reset_type.suppress_positive_response()
    }

    /// Getter for the requested [`ResetType`]
    pub fn reset_type(&self) -> ResetType {
        self.reset_type.value()
    }

    /// Get the allowed Nack codes for this request
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

    /// Serialization function to write a [`EcuResetRequest`] to a `Writer`
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.reset_type))?;
        Ok(1)
    }
}

impl SingleValueWireFormat for EcuResetRequest {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct EcuResetResponse {
    pub reset_type: ResetType,
    pub power_down_time: u8,
}

impl EcuResetResponse {
    /// Create a new 'EcuResetResponse'
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

    /// Serialization function to write a [`EcuResetResponse`] to a `Writer`
    fn to_writer<T: Write>(&self, buffer: &mut T) -> Result<usize, Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        buffer.write_u8(self.power_down_time)?;
        Ok(2)
    }
}

impl SingleValueWireFormat for EcuResetResponse {}
