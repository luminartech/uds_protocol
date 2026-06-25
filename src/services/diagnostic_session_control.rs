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

use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, NegativeResponseCode};

/// `DiagnosticSessionType` is used to specify or describe the session type of the server
///
/// *Note*:
///
/// Conversions from `u8` to `DiagnosticSessionType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticSessionType {
    /// This value is reserved by the ISO 14229-1 Specification
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u8),
    /// The `DefaultSession` (0x01) enables the standard diagnostic functionality
    /// - No `TesterPresent` messages are required to remain in this session
    /// - Any other diagnostic sessions are stopped upon successful entry into this session
    /// - Any security authorization is revoked
    /// - This session is initialized on startup
    DefaultSession,
    /// The `ProgrammingSession` (0x02) enables services required to support writing server memory
    /// - Upon timeout the server shall return to the `DefaultSession`
    /// - Success response may be sent before or after session is actually entered
    ProgrammingSession,
    /// The `ExtendedDiagnosticSession` (0x03) enables additional diagnostics functionality which can modify server behavior
    ExtendedDiagnosticSession,
    /// The `SafetySystemDiagnosticSession` (0x04) enables diagnostics functionality for safety systems
    SafetySystemDiagnosticSession,
    /// Value reserved for use by vehicle manufacturers
    #[cfg_attr(feature = "clap", clap(skip))]
    VehicleManufacturerSpecificSession(u8),
    /// Value reserved for use by system suppliers
    #[cfg_attr(feature = "clap", clap(skip))]
    SystemSupplierSpecificSession(u8),
}

impl From<DiagnosticSessionType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: DiagnosticSessionType) -> Self {
        match value {
            DiagnosticSessionType::ISOSAEReserved(value) => value,
            DiagnosticSessionType::DefaultSession => 0x01,
            DiagnosticSessionType::ProgrammingSession => 0x02,
            DiagnosticSessionType::ExtendedDiagnosticSession => 0x03,
            DiagnosticSessionType::SafetySystemDiagnosticSession => 0x04,
            DiagnosticSessionType::VehicleManufacturerSpecificSession(value) => value,
            DiagnosticSessionType::SystemSupplierSpecificSession(value) => value,
        }
    }
}

impl TryFrom<u8> for DiagnosticSessionType {
    type Error = Error;
    #[allow(clippy::match_same_arms)]
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(DiagnosticSessionType::ISOSAEReserved(value)),
            0x01 => Ok(DiagnosticSessionType::DefaultSession),
            0x02 => Ok(DiagnosticSessionType::ProgrammingSession),
            0x03 => Ok(DiagnosticSessionType::ExtendedDiagnosticSession),
            0x04 => Ok(DiagnosticSessionType::SafetySystemDiagnosticSession),
            0x05..=0x3F => Ok(DiagnosticSessionType::ISOSAEReserved(value)),
            0x40..=0x5F => Ok(DiagnosticSessionType::VehicleManufacturerSpecificSession(
                value,
            )),
            0x60..=0x7E => Ok(DiagnosticSessionType::SystemSupplierSpecificSession(value)),
            0x7F => Ok(DiagnosticSessionType::ISOSAEReserved(value)),
            _ => Err(Error::InvalidDiagnosticSessionType(value)),
        }
    }
}

#[cfg(test)]
mod diagnostic_session_type_tests {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn from_all_u8_values() {
        for i in 0..=u8::MAX {
            let msg_type = DiagnosticSessionType::try_from(i);
            match i {
                0x01 => assert!(matches!(
                    msg_type,
                    Ok(DiagnosticSessionType::DefaultSession)
                )),
                0x02 => assert!(matches!(
                    msg_type,
                    Ok(DiagnosticSessionType::ProgrammingSession)
                )),
                0x03 => assert!(matches!(
                    msg_type,
                    Ok(DiagnosticSessionType::ExtendedDiagnosticSession)
                )),
                0x04 => assert!(matches!(
                    msg_type,
                    Ok(DiagnosticSessionType::SafetySystemDiagnosticSession)
                )),
                0x00 | 0x05..=0x3F | 0x7F => {
                    assert!(matches!(
                        msg_type,
                        Ok(DiagnosticSessionType::ISOSAEReserved(_))
                    ));
                }
                0x40..=0x5F => {
                    assert!(matches!(
                        msg_type,
                        Ok(DiagnosticSessionType::VehicleManufacturerSpecificSession(_))
                    ));
                }
                0x60..=0x7E => {
                    assert!(matches!(
                        msg_type,
                        Ok(DiagnosticSessionType::SystemSupplierSpecificSession(_))
                    ));
                }
                _ => assert!(matches!(
                    msg_type,
                    Err(Error::InvalidDiagnosticSessionType(_))
                )),
            }
        }
    }

    #[test]
    fn from_all_enum_values() {
        assert_eq!(u8::from(DiagnosticSessionType::DefaultSession), 0x01);
        assert_eq!(u8::from(DiagnosticSessionType::ProgrammingSession), 0x02);
        assert_eq!(
            u8::from(DiagnosticSessionType::ExtendedDiagnosticSession),
            0x03
        );
        assert_eq!(
            u8::from(DiagnosticSessionType::SafetySystemDiagnosticSession),
            0x04
        );
    }
}

const DIAGNOSTIC_SESSION_CONTROL_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 3] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
];

/// Request for the server to change diagnostic session type
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct DiagnosticSessionControlRequest {
    session_type: SuppressablePositiveResponse<DiagnosticSessionType>,
}

impl DiagnosticSessionControlRequest {
    /// Create a new `DiagnosticSessionControlRequest`
    #[must_use]
    pub fn new(suppress_positive_response: bool, session_type: DiagnosticSessionType) -> Self {
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
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_diagnostic_session_control_request() {
        let bytes: [u8; 1] = [0x02];
        let (req, _) = <DiagnosticSessionControlRequest as Decode>::decode(&bytes).unwrap();
        assert!(!req.suppress_positive_response());
        assert_eq!(
            req.session_type(),
            DiagnosticSessionType::ProgrammingSession
        );

        let mut buffer = Vec::new();
        Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(req.encoded_size(), 1);
        assert_encode_size_agrees(&req);
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn test_diagnostic_session_control_response() {
        let bytes = [0x02, 0x11, 0x22, 0x33, 0x44];
        let (resp, _) = <DiagnosticSessionControlResponse as Decode>::decode(&bytes).unwrap();
        assert_eq!(resp.session_type, DiagnosticSessionType::ProgrammingSession);
        assert_eq!(resp.p2_server_max, 0x1122);
        assert_eq!(resp.p2_star_server_max, 0x3344);

        let mut buffer = Vec::new();
        Encode::encode(&resp, &mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(resp.encoded_size(), 5);
        assert_encode_size_agrees(&resp);
    }
}
