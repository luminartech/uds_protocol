use crate::{DtcSettings, Error, WireFormat, SUCCESS};
use byteorder::{ReadBytesExt, WriteBytesExt};

/// The ControlDTCSettings service is used to control the DTC settings of the ECU.
#[derive(Clone, Copy, Debug)]
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

impl WireFormat<Error> for ControlDTCSettingsRequest {
    /// ControlDTCSettingsRequest is not iterable
    const ITERABLE: bool = false;
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let request_byte = reader.read_u8()?;
        let setting = DtcSettings::from(request_byte & !SUCCESS);
        let suppress_response = request_byte & SUCCESS != 0;
        Ok(Some(Self {
            setting,
            suppress_response,
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let request_byte =
            u8::from(self.setting) | if self.suppress_response { SUCCESS } else { 0 };
        writer.write_u8(request_byte)?;
        Ok(1)
    }
}
