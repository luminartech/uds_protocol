//! `ECUReset` (0x11) service implementation
use crate::{
    Decode, Encode, Error, NegativeResponseCode, ResetType, SuppressablePositiveResponse,
};

const ECU_RESET_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::SecurityAccessDenied,
];

/// Request for the server to reset the ECU
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &ECU_RESET_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for EcuResetRequest {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.reset_type)])
            .map_err(Error::io)?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl<'a> Decode<'a> for EcuResetRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let reset_type = SuppressablePositiveResponse::try_from(buf[0])?;
        Ok((Self { reset_type }, &buf[1..]))
    }
}

/// Positive response to an `EcuResetRequest`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct EcuResetResponse {
    /// The reset type echoed from the request.
    pub reset_type: ResetType,
    /// Time in seconds before the server powers down (`0x00` = not available).
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

impl Encode for EcuResetResponse {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.reset_type), self.power_down_time])
            .map_err(Error::io)?;
        Ok(2)
    }
}

impl<'a> Decode<'a> for EcuResetResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let reset_type = ResetType::try_from(buf[0])?;
        // powerDownTime is conditional per ISO 14229-1
        let power_down_time = buf.get(1).copied().unwrap_or(0);
        let consumed = core::cmp::min(buf.len(), 2);
        Ok((
            Self {
                reset_type,
                power_down_time,
            },
            &buf[consumed..],
        ))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn ecu_reset_request() {
        let bytes: [u8; 2] = [0x81, 0x00];
        let req = EcuResetRequest::new(true, ResetType::HardReset);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        let (result, _) = <EcuResetRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(result, req);

        assert_eq!(written, 1);
        assert_eq!(written, req.encoded_size());
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn ecu_reset_response() {
        let bytes: [u8; 2] = [0x01, 0x20];
        let resp = EcuResetResponse::new(ResetType::HardReset, 0x20);
        let mut buffer = Vec::new();
        let written = Encode::encode(&resp, &mut buffer).unwrap();
        let (result, _) = <EcuResetResponse as Decode>::decode(&bytes).unwrap();
        assert_eq!(result, resp);

        assert_eq!(written, 2);
        assert_eq!(written, resp.encoded_size());
    }
}
