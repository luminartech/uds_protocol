//! The `DiagnosticSessionControl` service is used to enable different diagnostic sessions in the server.
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
    Decode, DiagnosticSessionType, Encode, Error, NegativeResponseCode, SuppressablePositiveResponse,
};

const DIAGNOSTIC_SESSION_CONTROL_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 3] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
];

/// Request for the server to change diagnostic session type
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticSessionControlRequest {
    session_type: SuppressablePositiveResponse<DiagnosticSessionType>,
}

impl DiagnosticSessionControlRequest {
    /// Create a new `DiagnosticSessionControlRequest`
    #[must_use] 
    pub fn new(
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
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.session_type.suppress_positive_response()
    }

    /// Getter for the requested session type
    #[must_use]
    pub fn session_type(&self) -> DiagnosticSessionType {
        self.session_type.value()
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &DIAGNOSTIC_SESSION_CONTROL_NEGATIVE_RESPONSE_CODES
    }
}
impl Encode for DiagnosticSessionControlRequest {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.session_type)])
            .map_err(Error::io)?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl<'a> Decode<'a> for DiagnosticSessionControlRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let session_type = SuppressablePositiveResponse::try_from(buf[0])?;
        Ok((Self { session_type }, &buf[1..]))
    }
}

/// Positive response to a `DiagnosticSessionControlRequest`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct DiagnosticSessionControlResponse {
    /// The session type that is now active.
    pub session_type: DiagnosticSessionType,
    /// P2 server max timing parameter (milliseconds).
    pub p2_server_max: u16,
    /// P2* (enhanced) server max timing parameter (milliseconds × 10).
    pub p2_star_server_max: u16,
}

impl DiagnosticSessionControlResponse {
    /// Create a new `DiagnosticSessionControlResponse`
    #[must_use] 
    pub fn new(
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
impl Encode for DiagnosticSessionControlResponse {
    fn encoded_size(&self) -> usize {
        5
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.session_type)])
            .map_err(Error::io)?;
        writer
            .write_all(&self.p2_server_max.to_be_bytes())
            .map_err(Error::io)?;
        writer
            .write_all(&self.p2_star_server_max.to_be_bytes())
            .map_err(Error::io)?;
        Ok(5)
    }
}

impl<'a> Decode<'a> for DiagnosticSessionControlResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 5 {
            return Err(Error::InsufficientData(5));
        }
        let session_type = DiagnosticSessionType::try_from(buf[0])?;
        let p2_server_max = u16::from_be_bytes([buf[1], buf[2]]);
        let p2_star_server_max = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((
            Self {
                session_type,
                p2_server_max,
                p2_star_server_max,
            },
            &buf[5..],
        ))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, DiagnosticSessionType, Encode};

    #[test]
    fn test_diagnostic_session_control_request() {
        let bytes: [u8; 1] = [0x02];
        let (req, _) =
            <DiagnosticSessionControlRequest as Decode>::decode(&bytes).unwrap();
        assert!(!req.suppress_positive_response());
        assert_eq!(
            req.session_type(),
            DiagnosticSessionType::ProgrammingSession
        );

        let mut buffer = Vec::new();
        Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(req.encoded_size(), 1);
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, DiagnosticSessionType, Encode};

    #[test]
    fn test_diagnostic_session_control_response() {
        let bytes = [0x02, 0x11, 0x22, 0x33, 0x44];
        let (resp, _) =
            <DiagnosticSessionControlResponse as Decode>::decode(&bytes).unwrap();
        assert_eq!(resp.session_type, DiagnosticSessionType::ProgrammingSession);
        assert_eq!(resp.p2_server_max, 0x1122);
        assert_eq!(resp.p2_star_server_max, 0x3344);

        let mut buffer = Vec::new();
        Encode::encode(&resp, &mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(resp.encoded_size(), 5);
    }
}
