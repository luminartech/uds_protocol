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
    DiagnosticSessionType, Error, NegativeResponseCode, SingleValueWireFormat,
    SuppressablePositiveResponse, WireFormat,
};
use byteorder_embedded_io::io::{ReadBytesExt, WriteBytesExt};

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
impl WireFormat for DiagnosticSessionControlRequest {
    fn required_size(&self) -> usize {
        1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.session_type))?;
        Ok(1)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl SingleValueWireFormat for DiagnosticSessionControlRequest {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let session_type = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        Ok(Self { session_type })
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
impl WireFormat for DiagnosticSessionControlResponse {
    fn required_size(&self) -> usize {
        5
    }

    fn encode<T: std::io::Write>(&self, buffer: &mut T) -> Result<usize, Error> {
        buffer.write_u8(u8::from(self.session_type))?;
        buffer.write_u16::<byteorder_embedded_io::BigEndian>(self.p2_server_max)?;
        buffer.write_u16::<byteorder_embedded_io::BigEndian>(self.p2_star_server_max)?;

        Ok(5)
    }
}

impl SingleValueWireFormat for DiagnosticSessionControlResponse {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let session_type = DiagnosticSessionType::try_from(reader.read_u8()?)?;
        let p2_server_max = reader.read_u16::<byteorder_embedded_io::BigEndian>()?;
        let p2_star_server_max = reader.read_u16::<byteorder_embedded_io::BigEndian>()?;
        Ok(Self {
            session_type,
            p2_server_max,
            p2_star_server_max,
        })
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::DiagnosticSessionType;
    use proptest::prelude::*;

    #[test]
    fn test_diagnostic_session_control_request() {
        let bytes: [u8; 1] = [0x02];
        let req: DiagnosticSessionControlRequest =
            DiagnosticSessionControlRequest::decode(&mut bytes.as_slice()).unwrap();
        assert!(!req.suppress_positive_response());
        assert_eq!(
            req.session_type(),
            DiagnosticSessionType::ProgrammingSession
        );

        let mut buffer = Vec::new();
        req.encode(&mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(req.required_size(), 1);
    }

    proptest! {
        #[test]
        fn prop_diagnostic_session_control_request_roundtrip(byte in 0x00u8..=0xFF) {
            // Full range: lower 7 bits encode session type, bit 7 is SPRMIB
            let req = DiagnosticSessionControlRequest::decode(&mut [byte].as_slice()).unwrap();
            let mut buf = Vec::new();
            req.encode(&mut buf).unwrap();
            let decoded = DiagnosticSessionControlRequest::decode(&mut buf.as_slice()).unwrap();
            prop_assert_eq!(req.session_type(), decoded.session_type());
            prop_assert_eq!(req.suppress_positive_response(), decoded.suppress_positive_response());
            // Verify SPRMIB bit is correctly interpreted
            prop_assert_eq!(req.suppress_positive_response(), byte & 0x80 != 0);
        }
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::DiagnosticSessionType;
    use proptest::prelude::*;

    #[test]
    fn test_diagnostic_session_control_response() {
        let bytes = [0x02, 0x11, 0x22, 0x33, 0x44];
        let resp: DiagnosticSessionControlResponse =
            DiagnosticSessionControlResponse::decode(&mut bytes.as_slice()).unwrap();
        assert_eq!(resp.session_type, DiagnosticSessionType::ProgrammingSession);
        assert_eq!(resp.p2_server_max, 0x1122);
        assert_eq!(resp.p2_star_server_max, 0x3344);

        let mut buffer = Vec::new();
        resp.encode(&mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(resp.required_size(), 5);
    }

    proptest! {
        #[test]
        fn prop_diagnostic_session_control_response_roundtrip(
            session_byte in 0x01u8..=0x04,
            p2 in any::<u16>(),
            p2_star in any::<u16>(),
        ) {
            let session = DiagnosticSessionType::try_from(session_byte).unwrap();
            let resp = DiagnosticSessionControlResponse::new(session, p2, p2_star);
            let mut buf = Vec::new();
            resp.encode(&mut buf).unwrap();

            let decoded = DiagnosticSessionControlResponse::decode(&mut buf.as_slice()).unwrap();
            prop_assert_eq!(resp, decoded);
        }
    }
}
