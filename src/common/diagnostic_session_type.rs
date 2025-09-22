use clap::ValueEnum;

use crate::Error;

/// `DiagnosticSessionType` is used to specify or describe the session type of the server
///
/// *Note*:
///
/// Conversions from `u8` to `DiagnosticSessionType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq,  ValueEnum)]
pub enum DiagnosticSessionType {
    /// This value is reserved by the ISO 14229-1 Specification
    #[clap(skip)]
    ISOSAEReserved(u8),
    /// The `DefaultSession` (0x01) enables the standard diagnostic functionality
    /// - No `TesterPresent` messages are required to remain in this session
    /// - Any other diagnostic sessions are stopped upon succesful entry into this session
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
    #[clap(skip)]
    VehicleManufacturerSpecificSession(u8),
    /// Value reserved for use by system suppliers
    #[clap(skip)]
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
mod test {
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
