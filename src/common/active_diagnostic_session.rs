use crate::{DiagnosticSessionType, Error, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use utoipa::ToSchema;

/// Represents the active diagnostic session of the lidar module.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, Parser, ToSchema)]
pub struct ActiveDiagnosticSession {
    /// The current diagnostic session type.
    pub current_session: DiagnosticSessionType,
}

impl ActiveDiagnosticSession {
    /// Creates a new `ActiveDiagnosticSession` instance.
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

    fn to_writer<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let value = u8::from(self.current_session);
        writer.write_u8(value)?;
        Ok(1)
    }
}

impl SingleValueWireFormat for ActiveDiagnosticSession {}
