use crate::{DtcSettings, Error, SUCCESS, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The ControlDTCSettings service is used to control the DTC settings of the ECU.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
#[non_exhaustive]
pub struct ControlDTCSettingsRequest {
    /// The requested DTC logging setting
    pub setting: DtcSettings,
    /// Whether the ECU should suppress a response
    pub suppress_response: bool,
}

impl ControlDTCSettingsRequest {
    pub(crate) fn new(setting: DtcSettings, suppress_response: bool) -> Self {
        Self {
            setting,
            suppress_response,
        }
    }
}

impl WireFormat for ControlDTCSettingsRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let request_byte = reader.read_u8()?;
        let setting = DtcSettings::from(request_byte & !SUCCESS);
        let suppress_response = request_byte & SUCCESS != 0;
        Ok(Some(Self {
            setting,
            suppress_response,
        }))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let request_byte =
            u8::from(self.setting) | if self.suppress_response { SUCCESS } else { 0 };
        writer.write_u8(request_byte)?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_response
    }
}

impl SingleValueWireFormat for ControlDTCSettingsRequest {}

/// Positive response to a ControlDTCSettingsRequest
///
/// The ECU will respond with a ControlDTCSettingsResponse if the request was successful.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
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
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let setting = DtcSettings::from(reader.read_u8()?);
        Ok(Some(Self { setting }))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
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
        let req = ControlDTCSettingsRequest::new(DtcSettings::On, true);
        let mut buffer = Vec::new();
        let written = req.to_writer(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.required_size(), buffer.len());

        let parsed = ControlDTCSettingsRequest::from_reader(&mut buffer.as_slice()).unwrap();
        assert_eq!(parsed.setting, DtcSettings::On);
        assert!(parsed.suppress_response);
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
        let written = req.to_writer(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.required_size(), buffer.len());

        let parsed = ControlDTCSettingsResponse::from_reader(&mut buffer.as_slice()).unwrap();
        assert_eq!(parsed.setting, DtcSettings::On);
    }
}
