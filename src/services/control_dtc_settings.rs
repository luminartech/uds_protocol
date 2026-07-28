//! `ControlDTCSetting` (0x85) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};
use automotive_wire_codec::write_u8;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
/// Controls whether the server should enable or disable DTC status-bit updates.
///
/// Used by [`ControlDtcSettingRequest`] to instruct the server.
pub enum DtcSettingType {
    /// Re-enable DTC status-bit updates.
    On,
    /// Disable DTC status-bit updates.
    Off,
}

impl From<DtcSettingType> for u8 {
    fn from(value: DtcSettingType) -> Self {
        match value {
            DtcSettingType::On => 0x01,
            DtcSettingType::Off => 0x02,
        }
    }
}

impl TryFrom<u8> for DtcSettingType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Off),
            _ => Err(Error::InvalidDtcSetting(value)),
        }
    }
}

/// The `ControlDtcSetting` service is used to control the DTC settings of the ECU.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDtcSettingRequest {
    /// Whether the server should suppress the positive response (SPRMIB).
    pub suppress_positive_response: bool,
    /// The requested DTC logging setting.
    pub setting: DtcSettingType,
}

const CONTROL_DTC_SETTING_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
];

impl ControlDtcSettingRequest {
    /// Create a new `ControlDtcSettingRequest`.
    #[must_use]
    pub const fn new(suppress_positive_response: bool, setting: DtcSettingType) -> Self {
        Self {
            suppress_positive_response,
            setting,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &CONTROL_DTC_SETTING_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ControlDtcSettingRequest {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sub_function =
            SuppressablePositiveResponse::new(self.suppress_positive_response, self.setting);
        write_u8(writer, u8::from(sub_function)).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for ControlDtcSettingRequest {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let sub_function = SuppressablePositiveResponse::<DtcSettingType>::try_from(buf[0])?;
        Ok((
            Self {
                suppress_positive_response: sub_function.suppress_positive_response(),
                setting: sub_function.value(),
            },
            &buf[1..],
        ))
    }
}

/// Positive response to a `ControlDtcSettingRequest`
///
/// The ECU will respond with a `ControlDtcSettingResponse` if the request was successful.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDtcSettingResponse {
    /// The DTC logging setting that was set in the request
    pub setting: DtcSettingType,
}

impl ControlDtcSettingResponse {
    /// Create a new `ControlDtcSettingResponse`.
    #[must_use]
    pub const fn new(setting: DtcSettingType) -> Self {
        Self { setting }
    }
}

impl Encode for ControlDtcSettingResponse {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, u8::from(self.setting)).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for ControlDtcSettingResponse {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let setting = DtcSettingType::try_from(buf[0])?;
        Ok((Self { setting }, &buf[1..]))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, NegativeResponseCode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::{vec, vec::Vec};

    #[cfg(feature = "alloc")]
    #[test]
    fn simple_request() {
        let req = ControlDtcSettingRequest::new(true, DtcSettingType::On);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.encoded_size().unwrap(), buffer.len());

        let (parsed, _) = <ControlDtcSettingRequest as Decode>::decode(&buffer).unwrap();
        assert_eq!(parsed.setting, DtcSettingType::On);
        assert!(parsed.suppress_positive_response);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn invalid_setting_byte_carries_the_value() {
        // An unrecognized setting must surface the offending byte, like every other
        // service's Invalid<Service>Type error, not the generic length/format error.
        let err = <ControlDtcSettingRequest as Decode>::decode(&[0x09]).unwrap_err();
        assert!(matches!(err, Error::InvalidDtcSetting(0x09)));
    }

    #[test]
    fn exposes_allowed_nack_codes() {
        assert!(!ControlDtcSettingRequest::allowed_nack_codes().is_empty());
        assert!(
            ControlDtcSettingRequest::allowed_nack_codes()
                .contains(&NegativeResponseCode::RequestOutOfRange)
        );
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::{vec, vec::Vec};

    #[cfg(feature = "alloc")]
    #[test]
    fn simple_response() {
        let req = ControlDtcSettingResponse::new(DtcSettingType::On);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.encoded_size().unwrap(), buffer.len());

        let (parsed, _) = <ControlDtcSettingResponse as Decode>::decode(&buffer).unwrap();
        assert_eq!(parsed.setting, DtcSettingType::On);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn response_is_eq() {
        crate::test_util::assert_impl_eq::<ControlDtcSettingResponse>();
    }
}
