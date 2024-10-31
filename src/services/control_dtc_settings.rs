use crate::{DtcSettings, Error, SUCCESS};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// The ControlDTCSettings service is used to control the DTC settings of the ECU.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct ControlDTCSettings {
    /// The requested DTC logging setting
    pub setting: DtcSettings,
    /// Whether the ECU should suppress a response
    pub suppress_response: bool,
}

impl ControlDTCSettings {
    pub(crate) fn new(setting: DtcSettings, suppress_response: bool) -> Self {
        Self {
            setting,
            suppress_response,
        }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let request_byte = buffer.read_u8()?;
        let setting = DtcSettings::from(request_byte & !SUCCESS);
        let suppress_response = request_byte & SUCCESS != 0;
        Ok(Self {
            setting,
            suppress_response,
        })
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        let request_byte =
            u8::from(self.setting) | if self.suppress_response { SUCCESS } else { 0 };
        buffer.write_u8(request_byte)?;
        Ok(())
    }
}
