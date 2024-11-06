use crate::{DiagnosticSessionType, Error, SuppressablePositiveResponse};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Request for the server to change diagnostic session type
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiagnosticSessionControlRequest {
    session_type: SuppressablePositiveResponse<DiagnosticSessionType>,
}

impl DiagnosticSessionControlRequest {
    /// Create a new DiagnosticSessionControlRequest
    pub(crate) fn new(
        suppress_positive_response: bool,
        session_type: DiagnosticSessionType,
    ) -> Self {
        Self {
            session_type: SuppressablePositiveResponse::new(
                suppress_positive_response,
                session_type,
            ),
        }
    }

    /// Getter for whether a positive response should be suppressed
    pub fn suppress_positive_response(&self) -> bool {
        self.session_type.suppress_positive_response()
    }

    /// Getter for the requested session type
    pub fn session_type(&self) -> DiagnosticSessionType {
        self.session_type.value()
    }

    /// Deserialization function to read a DiagnosticSessionControlRequest from a `Reader`
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let session_type = SuppressablePositiveResponse::from(buffer.read_u8()?);
        Ok(Self { session_type })
    }

    /// Serialization function to write a DiagnosticSessionControlRequest to a `Writer`
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.session_type))?;
        Ok(())
    }
}

/// Positive response to a DiagnosticSessionControlRequest
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct DiagnosticSessionControlResponse {
    pub session_type: DiagnosticSessionType,
    pub p2_server_max: u16,
    pub p2_star_server_max: u16,
}

impl DiagnosticSessionControlResponse {
    /// Create a new DiagnosticSessionControlResponse
    pub(crate) fn new(
        session_type: DiagnosticSessionType,
        p2_server_max: u16,
        p2_star_server_max: u16,
    ) -> Self {
        Self {
            session_type,
            p2_server_max,
            p2_star_server_max,
        }
    }

    /// Read a DiagnosticSessionControlResponse from a `Reader`
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let session_type = DiagnosticSessionType::from(buffer.read_u8()?);
        let p2_server_max = buffer.read_u16::<byteorder::BigEndian>()?;
        let p2_star_server_max = buffer.read_u16::<byteorder::BigEndian>()?;
        Ok(Self {
            session_type,
            p2_server_max,
            p2_star_server_max,
        })
    }

    /// Write a DiagnosticSessionControlResponse to a `Writer`
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.session_type))?;
        buffer.write_u16::<byteorder::BigEndian>(self.p2_server_max)?;
        buffer.write_u16::<byteorder::BigEndian>(self.p2_star_server_max)?;
        Ok(())
    }
}
