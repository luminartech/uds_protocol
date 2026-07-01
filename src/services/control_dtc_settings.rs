//! `ControlDTCSetting` (0x85) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, NegativeResponseCode};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
/// Controls whether the server should enable or disable DTC status-bit updates.
///
/// Used by [`ControlDTCSettingsRequest`] to instruct the server.
pub enum DtcSettings {
    /// Re-enable DTC status-bit updates.
    On,
    /// Disable DTC status-bit updates.
    Off,
}

impl From<DtcSettings> for u8 {
    fn from(value: DtcSettings) -> Self {
        match value {
            DtcSettings::On => 0x01,
            DtcSettings::Off => 0x02,
        }
    }
}

impl TryFrom<u8> for DtcSettings {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Off),
            _ => Err(Error::InvalidDtcSetting(value)),
        }
    }
}

/// The `ControlDTCSettings` service is used to control the DTC settings of the ECU.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDTCSettingsRequest {
    /// Whether the server should suppress the positive response (SPRMIB).
    pub suppress_positive_response: bool,
    /// The requested DTC logging setting.
    pub setting: DtcSettings,
}

const CONTROL_DTC_SETTINGS_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
];

impl ControlDTCSettingsRequest {
    /// Create a new `ControlDTCSettingsRequest`.
    #[must_use]
    pub const fn new(suppress_positive_response: bool, setting: DtcSettings) -> Self {
        Self {
            suppress_positive_response,
            setting,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &CONTROL_DTC_SETTINGS_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ControlDTCSettingsRequest {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sub_function =
            SuppressablePositiveResponse::new(self.suppress_positive_response, self.setting);
        writer
            .write_all(&[u8::from(sub_function)])
            .map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for ControlDTCSettingsRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let sub_function = SuppressablePositiveResponse::<DtcSettings>::try_from(buf[0])?;
        Ok((
            Self {
                suppress_positive_response: sub_function.suppress_positive_response(),
                setting: sub_function.value(),
            },
            &buf[1..],
        ))
    }
}

/// Positive response to a `ControlDTCSettingsRequest`
///
/// The ECU will respond with a `ControlDTCSettingsResponse` if the request was successful.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDTCSettingsResponse {
    /// The DTC logging setting that was set in the request
    pub setting: DtcSettings,
}

impl ControlDTCSettingsResponse {
    /// Create a new `ControlDTCSettingsResponse`.
    #[must_use]
    pub const fn new(setting: DtcSettings) -> Self {
        Self { setting }
    }
}

impl Encode for ControlDTCSettingsResponse {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.setting)])
            .map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for ControlDTCSettingsResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let setting = DtcSettings::try_from(buf[0])?;
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
        let req = ControlDTCSettingsRequest::new(true, DtcSettings::On);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.encoded_size(), buffer.len());

        let (parsed, _) = <ControlDTCSettingsRequest as Decode>::decode(&buffer).unwrap();
        assert_eq!(parsed.setting, DtcSettings::On);
        assert!(parsed.suppress_positive_response);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn invalid_setting_byte_carries_the_value() {
        // An unrecognized setting must surface the offending byte, like every other
        // service's Invalid<Service>Type error, not the generic length/format error.
        let err = <ControlDTCSettingsRequest as Decode>::decode(&[0x09]).unwrap_err();
        assert!(matches!(err, Error::InvalidDtcSetting(0x09)));
    }

    #[test]
    fn exposes_allowed_nack_codes() {
        assert!(!ControlDTCSettingsRequest::allowed_nack_codes().is_empty());
        assert!(ControlDTCSettingsRequest::allowed_nack_codes()
            .contains(&NegativeResponseCode::RequestOutOfRange));
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
        let req = ControlDTCSettingsResponse::new(DtcSettings::On);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.encoded_size(), buffer.len());

        let (parsed, _) = <ControlDTCSettingsResponse as Decode>::decode(&buffer).unwrap();
        assert_eq!(parsed.setting, DtcSettings::On);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn response_is_eq() {
        crate::test_util::assert_impl_eq::<ControlDTCSettingsResponse>();
    }
}
