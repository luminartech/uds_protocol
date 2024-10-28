use crate::{CommunicationEnable, CommunicationType, Error, SUCCESS};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct CommunicationControl {
    pub communication_enable: CommunicationEnable,
    pub communication_type: CommunicationType,
    pub suppress_response: bool,
    /// Stop external code from creating instances of this struct directly
    _private: (),
}

impl CommunicationControl {
    pub(crate) fn new(
        communication_enable: CommunicationEnable,
        communication_type: CommunicationType,
        suppress_response: bool,
    ) -> Self {
        Self {
            communication_enable,
            communication_type,
            suppress_response,
            _private: (),
        }
    }
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let enable_byte = buffer.read_u8()?;
        let communication_enable = CommunicationEnable::from(enable_byte & !SUCCESS);
        let suppress_response = enable_byte & SUCCESS == SUCCESS;
        let communication_type = CommunicationType::from(buffer.read_u8()?);
        Ok(Self {
            communication_enable,
            communication_type,
            suppress_response,
            _private: (),
        })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        let communication_enable_byte =
            u8::from(self.communication_enable) | if self.suppress_response { SUCCESS } else { 0 };
        buffer.write_u8(communication_enable_byte)?;
        buffer.write_u8(u8::from(self.communication_type))?;
        Ok(())
    }
}
