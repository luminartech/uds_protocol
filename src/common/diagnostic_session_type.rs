use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use super::suppressable_positive_response::SPRMIB_VALUE_MASK;

/// `DiagnosticSessionType` is used to specify or describe the session type of the server
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum DiagnosticSessionType {
    /// This value is reserved by the ISO 14229-1 Specification
    #[clap(skip)]
    ISOSAEReserved(u8),
    /// The `DefaultSession` enables the standard diagnostic functionality
    /// - No TesterPresent messages are required to remain in this session
    /// - Any other diagnostic sessions are stopped upon succesful entry into this session
    /// - Any security authorization is revoked
    /// - This session is initialized on startup
    DefaultSession,
    /// The `ProgrammingSession` enables services required to support writing server memory
    /// - Upon timeout the server shall return to the `DefaultSession`
    /// - Success response may be sent before or after session is actually entered
    ProgrammingSession,
    /// The `ExtendedDiagnosticSession` enables additional diagnostics functionality which can modify server behavior
    ExtendedDiagnosticSession,
    /// The `SafetySystemDiagnosticSession` enables diagnostics functionality for safety systems
    SafetySystemDiagnosticSession,
    /// Value reserved for use by vehicle manufacturers
    #[clap(skip)]
    VehicleManufacturerSpecificSession(u8),
    /// Value reserved for use by system suppliers
    #[clap(skip)]
    SystemSupplierSpecificSession(u8),
}

impl From<DiagnosticSessionType> for u8 {
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

impl From<u8> for DiagnosticSessionType {
    fn from(value: u8) -> Self {
        let value = value & SPRMIB_VALUE_MASK;
        match value {
            0x00 => DiagnosticSessionType::ISOSAEReserved(value),
            0x01 => DiagnosticSessionType::DefaultSession,
            0x02 => DiagnosticSessionType::ProgrammingSession,
            0x03 => DiagnosticSessionType::ExtendedDiagnosticSession,
            0x04 => DiagnosticSessionType::SafetySystemDiagnosticSession,
            0x05..=0x3F => DiagnosticSessionType::ISOSAEReserved(value),
            0x40..=0x5F => DiagnosticSessionType::VehicleManufacturerSpecificSession(value),
            0x60..=0x7E => DiagnosticSessionType::SystemSupplierSpecificSession(value),
            0x7F => DiagnosticSessionType::ISOSAEReserved(value),
            _ => unreachable!("This code cannot be reached because the SPRMIB has been masked off"),
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
            let msg_type = DiagnosticSessionType::from(i);
            match i {
                0x01 => assert_eq!(msg_type, DiagnosticSessionType::DefaultSession),
                0x02 => assert_eq!(msg_type, DiagnosticSessionType::ProgrammingSession),
                0x03 => assert_eq!(msg_type, DiagnosticSessionType::ExtendedDiagnosticSession),
                0x04 => assert_eq!(
                    msg_type,
                    DiagnosticSessionType::SafetySystemDiagnosticSession
                ),
                0x00 | 0x05..=0x3F | 0x7F => {
                    assert_eq!(msg_type, DiagnosticSessionType::ISOSAEReserved(i))
                }
                0x40..=0x5F => {
                    assert_eq!(
                        msg_type,
                        DiagnosticSessionType::VehicleManufacturerSpecificSession(i)
                    )
                }
                0x60..=0x7E => {
                    assert_eq!(
                        msg_type,
                        DiagnosticSessionType::SystemSupplierSpecificSession(i)
                    )
                }
                _ => assert_eq!(msg_type, DiagnosticSessionType::from(i & SPRMIB_VALUE_MASK)),
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
