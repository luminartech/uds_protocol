use crate::{DiagnosticSessionType, Error, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Represents the active diagnostic session of the lidar module.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::Parser))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActiveDiagnosticSession {
    /// The current diagnostic session type.
    pub current_session: DiagnosticSessionType,
}

impl ActiveDiagnosticSession {
    /// Creates a new `ActiveDiagnosticSession` instance.
    ///
    /// # Errors
    /// Will return error of type `Error::InvalidDiagnosticSessionType` if `current_session` value is > 0x7F
    ///
    pub fn new(current_session: u8) -> Result<Self, Error> {
        Ok(ActiveDiagnosticSession {
            current_session: current_session.try_into()?,
        })
    }
}

impl WireFormat for ActiveDiagnosticSession {
    fn option_from_reader<R: Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let value = reader.read_u8()?;
        Ok(Some(ActiveDiagnosticSession::new(value)?))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let value = u8::from(self.current_session);
        writer.write_u8(value)?;
        Ok(1)
    }
}

impl SingleValueWireFormat for ActiveDiagnosticSession {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_diagnostic_session() {
        let bytes = [0x01]; // DiagnosticSessionType::DefaultSession
        let mut reader = &bytes[..];

        let response = ActiveDiagnosticSession::option_from_reader(&mut reader)
            .unwrap()
            .unwrap();

        assert_eq!(
            response.current_session,
            DiagnosticSessionType::DefaultSession
        );
        assert_eq!(response.required_size(), 1);

        let mut writer = Vec::new();
        let written = response.encode(&mut writer).unwrap();
        assert_eq!(writer, bytes, "Written: \n{writer:02X?}\n{bytes:02X?}");
        assert_eq!(written, bytes.len(), "Written: \n{writer:?}\n{bytes:?}");
        assert_eq!(written, response.required_size());
    }
}
