use crate::{DiagnosticSessionType, Error, SuppressablePositiveResponse};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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
