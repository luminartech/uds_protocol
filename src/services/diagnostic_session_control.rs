//! The DiagnosticSessionControl service is used to enable different diagnostic sessions in the server.
//! A diagnostic session enables a specific set of diagnostic services and/or functionality in the server.
//! This service provides the capability that the server can report data link layer specific parameter
//! values valid for the enabled diagnostic session (e.g. timing parameter values).
//! The user of this document shall define the exact set of services and/or functionality enabled in each diagnostic session.
//! There shall always be exactly one diagnostic session active in a server.
//! A server shall always start the default diagnostic session when powered up.
//! If no other diagnostic session is started, then the default diagnostic session shall be running as long as the server is powered.
//! A server shall be capable of providing diagnostic functionality under normal operating conditions,
//! as well as in other operation conditions defined by the vehicle manufacturer (e.g. limp home operation condition).

use crate::{
    DiagnosticSessionType, Error, NegativeResponseCode, SingleValueWireFormat,
    SuppressablePositiveResponse, WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

const DIAGNOSTIC_SESSION_CONTROL_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 3] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
];

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

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &DIAGNOSTIC_SESSION_CONTROL_NEGATIVE_RESPONSE_CODES
    }
}
impl WireFormat<'_, Error> for DiagnosticSessionControlRequest {
    /// Deserialization function to read a DiagnosticSessionControlRequest from a `Reader`
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let session_type = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        Ok(Some(Self { session_type }))
    }

    /// Serialization function to write a DiagnosticSessionControlRequest to a `Writer`
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.session_type))?;
        Ok(1)
    }
}

impl SingleValueWireFormat<'_, Error> for DiagnosticSessionControlRequest {}

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
}
impl WireFormat<'_, Error> for DiagnosticSessionControlResponse {
    /// Read a DiagnosticSessionControlResponse from a `Reader`
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let session_type = DiagnosticSessionType::try_from(reader.read_u8()?)?;
        let p2_server_max = reader.read_u16::<byteorder::BigEndian>()?;
        let p2_star_server_max = reader.read_u16::<byteorder::BigEndian>()?;
        Ok(Some(Self {
            session_type,
            p2_server_max,
            p2_star_server_max,
        }))
    }

    /// Write a DiagnosticSessionControlResponse to a `Writer`
    fn to_writer<T: std::io::Write>(&self, buffer: &mut T) -> Result<usize, Error> {
        buffer.write_u8(u8::from(self.session_type))?;
        buffer.write_u16::<byteorder::BigEndian>(self.p2_server_max)?;
        buffer.write_u16::<byteorder::BigEndian>(self.p2_star_server_max)?;
        Ok(5)
    }
}

impl SingleValueWireFormat<'_, Error> for DiagnosticSessionControlResponse {}
