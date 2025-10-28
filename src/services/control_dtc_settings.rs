use crate::{DtcSettings, Error, SingleValueWireFormat, SuppressablePositiveResponse, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};

/// The `ControlDTCSettings` service is used to control the DTC settings of the ECU.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ControlDTCSettingsRequest {
    /// The requested DTC logging setting
    setting: SuppressablePositiveResponse<DtcSettings>,
}

impl ControlDTCSettingsRequest {
    pub(crate) fn new(suppress_positive_response: bool, setting: DtcSettings) -> Self {
        Self {
            setting: SuppressablePositiveResponse::new(suppress_positive_response, setting),
        }
    }
    /// Getter for whether a positive response should be suppressed
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.setting.suppress_positive_response()
    }

    /// Getter for the setting
    #[must_use]
    pub fn setting(&self) -> DtcSettings {
        self.setting.value()
    }
}

impl WireFormat for ControlDTCSettingsRequest {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let setting = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        Ok(Some(Self { setting }))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.setting))?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl SingleValueWireFormat for ControlDTCSettingsRequest {}

/// Positive response to a `ControlDTCSettingsRequest`
///
/// The ECU will respond with a `ControlDTCSettingsResponse` if the request was successful.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ControlDTCSettingsResponse {
    /// The DTC logging setting that was set in the request
    pub setting: DtcSettings,
}

impl ControlDTCSettingsResponse {
    pub(crate) fn new(setting: DtcSettings) -> Self {
        Self { setting }
    }
}

impl WireFormat for ControlDTCSettingsResponse {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let setting = DtcSettings::try_from(reader.read_u8()?)?;
        Ok(Some(Self { setting }))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.setting))?;
        Ok(1)
    }
}

impl SingleValueWireFormat for ControlDTCSettingsResponse {}

#[cfg(test)]
mod request {
    use super::*;
    use crate::DtcSettings;

    #[test]
    fn simple_request() {
        let req = ControlDTCSettingsRequest::new(true, DtcSettings::On);
        let mut buffer = Vec::new();
        let written = req.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.required_size(), buffer.len());

        let parsed =
            ControlDTCSettingsRequest::decode_single_value(&mut buffer.as_slice()).unwrap();
        assert_eq!(parsed.setting(), DtcSettings::On);
        assert!(parsed.suppress_positive_response());
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::DtcSettings;

    #[test]
    fn simple_response() {
        let req = ControlDTCSettingsResponse::new(DtcSettings::On);
        let mut buffer = Vec::new();
        let written = req.encode(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.required_size(), buffer.len());

        let parsed =
            ControlDTCSettingsResponse::decode_single_value(&mut buffer.as_slice()).unwrap();
        assert_eq!(parsed.setting, DtcSettings::On);
    }
}
