//! `ControlDTCSetting` (0x85) service implementation
use crate::{Decode, DtcSettings, Encode, Error, SUCCESS, SingleValueWireFormat, WireFormat};
use byteorder_embedded_io::io::{ReadBytesExt, WriteBytesExt};

/// The `ControlDTCSettings` service is used to control the DTC settings of the ECU.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
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

impl Encode for ControlDTCSettingsRequest {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let request_byte =
            u8::from(self.setting) | if self.suppress_response { SUCCESS } else { 0 };
        writer.write_all(&[request_byte]).map_err(Error::io)?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_response
    }
}

impl<'a> Decode<'a> for ControlDTCSettingsRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let request_byte = buf[0];
        let setting = DtcSettings::try_from(request_byte & !SUCCESS)?;
        let suppress_response = request_byte & SUCCESS != 0;
        Ok((
            Self {
                setting,
                suppress_response,
            },
            &buf[1..],
        ))
    }
}

impl WireFormat for ControlDTCSettingsRequest {
    fn required_size(&self) -> usize {
        1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let request_byte =
            u8::from(self.setting) | if self.suppress_response { SUCCESS } else { 0 };
        writer.write_u8(request_byte)?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_response
    }
}

impl SingleValueWireFormat for ControlDTCSettingsRequest {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let request_byte = reader.read_u8()?;
        let setting = DtcSettings::try_from(request_byte & !SUCCESS)?;
        let suppress_response = request_byte & SUCCESS != 0;
        Ok(Self {
            setting,
            suppress_response,
        })
    }
}

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

impl WireFormat for ControlDTCSettingsResponse {
    fn required_size(&self) -> usize {
        1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.setting))?;
        Ok(1)
    }
}

impl SingleValueWireFormat for ControlDTCSettingsResponse {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let setting = DtcSettings::try_from(reader.read_u8()?)?;
        Ok(Self { setting })
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::DtcSettings;

    #[test]
    fn simple_request() {
        let req = ControlDTCSettingsRequest::new(DtcSettings::On, true);
        let mut buffer = Vec::new();
        let written = WireFormat::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.required_size(), buffer.len());

        let parsed = <ControlDTCSettingsRequest as SingleValueWireFormat>::decode(&mut buffer.as_slice()).unwrap();
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
        let written = WireFormat::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.required_size(), buffer.len());

        let parsed = <ControlDTCSettingsResponse as SingleValueWireFormat>::decode(&mut buffer.as_slice()).unwrap();
        assert_eq!(parsed.setting, DtcSettings::On);
    }
}
