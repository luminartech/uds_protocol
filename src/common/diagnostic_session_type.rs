use crate::Error;

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
#[repr(u8)]
pub enum DiagnosticSessionType {
    /// This value is reserved by the ISO 14229-1 Specification
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u8),
    /// The `DefaultSession` (0x01) enables the standard diagnostic functionality
    /// - No `TesterPresent` messages are required to remain in this session
    /// - Any other diagnostic sessions are stopped upon succesful entry into this session
    /// - Any security authorization is revoked
    /// - This session is initialized on startup
    DefaultSession = Self::DEFAULT_SESSION,
    /// The `ProgrammingSession` (0x02) enables services required to support writing server memory
    /// - Upon timeout the server shall return to the `DefaultSession`
    /// - Success response may be sent before or after session is actually entered
    ProgrammingSession = Self::PROGRAMMING_SESSION,
    /// The `ExtendedDiagnosticSession` (0x03) enables additional diagnostics functionality which can modify server behavior
    ExtendedDiagnosticSession = Self::EXTENDED_SESSION,
    /// The `SafetySystemDiagnosticSession` (0x04) enables diagnostics functionality for safety systems
    SafetySystemDiagnosticSession = Self::SAFETY_SYSTEM_SESSION,
    /// Value reserved for use by vehicle manufacturers
    #[cfg_attr(feature = "clap", clap(skip))]
    VehicleManufacturerSpecificSession(u8),
    /// Value reserved for use by system suppliers
    #[cfg_attr(feature = "clap", clap(skip))]
    SystemSupplierSpecificSession(u8),
}

impl DiagnosticSessionType {
    pub const ISO_RESERVED: u8 = 0x00;
    pub const DEFAULT_SESSION: u8 = 0x01;
    pub const PROGRAMMING_SESSION: u8 = 0x02;
    pub const EXTENDED_SESSION: u8 = 0x03;
    pub const SAFETY_SYSTEM_SESSION: u8 = 0x04;
    pub const RESERVED_START: u8 = 0x05;
    pub const RESERVED_END: u8 = 0x3F;
    pub const VEHICLE_MANUFACTURER_START: u8 = 0x40;
    pub const VEHICLE_MANUFACTURER_END: u8 = 0x5F;
    pub const SYSTEM_SUPPLIER_START: u8 = 0x60;
    pub const SYSTEM_SUPPLIER_END: u8 = 0x7E;
    pub const ISO_RESERVED_EXTENSION: u8 = 0x7F;
}

impl From<DiagnosticSessionType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: DiagnosticSessionType) -> Self {
        match value {
            DiagnosticSessionType::ISOSAEReserved(value) => value,
            DiagnosticSessionType::DefaultSession => DiagnosticSessionType::DEFAULT_SESSION,
            DiagnosticSessionType::ProgrammingSession => DiagnosticSessionType::PROGRAMMING_SESSION,
            DiagnosticSessionType::ExtendedDiagnosticSession => {
                DiagnosticSessionType::EXTENDED_SESSION
            }
            DiagnosticSessionType::SafetySystemDiagnosticSession => {
                DiagnosticSessionType::SAFETY_SYSTEM_SESSION
            }
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
            Self::ISO_RESERVED => Ok(DiagnosticSessionType::ISOSAEReserved(value)),
            Self::DEFAULT_SESSION => Ok(DiagnosticSessionType::DefaultSession),
            Self::PROGRAMMING_SESSION => Ok(DiagnosticSessionType::ProgrammingSession),
            Self::EXTENDED_SESSION => Ok(DiagnosticSessionType::ExtendedDiagnosticSession),
            Self::SAFETY_SYSTEM_SESSION => Ok(DiagnosticSessionType::SafetySystemDiagnosticSession),
            Self::RESERVED_START..=Self::RESERVED_END => {
                Ok(DiagnosticSessionType::ISOSAEReserved(value))
            }
            Self::VEHICLE_MANUFACTURER_START..=Self::VEHICLE_MANUFACTURER_END => Ok(
                DiagnosticSessionType::VehicleManufacturerSpecificSession(value),
            ),
            Self::SYSTEM_SUPPLIER_START..=Self::SYSTEM_SUPPLIER_END => {
                Ok(DiagnosticSessionType::SystemSupplierSpecificSession(value))
            }
            Self::ISO_RESERVED_EXTENSION => Ok(DiagnosticSessionType::ISOSAEReserved(value)),
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
