//! DIDs are used to identify the data that is requested or sent in a diagnostic service.
use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UDSIdentifier {
    ISOSAEReserved(u16),
    BootSoftwareIdentification,
    ApplicationSoftware,
    ApplicationDataIdentification,
    BootSoftwareFingerprint,
    ApplicationSoftwareFingerprint,
    ApplicationDataFingerprint,
    ActiveDiagnosticSession,
    VehicleManufacturerSparePartNumber,
    VehicleManufacturerECUSoftwareNumber,
    VehicleManufacturerECUSoftwareVersionNumber,
}

impl TryFrom<u16> for UDSIdentifier {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0000..=0x00FF => Self::ISOSAEReserved(value),
            // 0x0100..0xA5FF => Manufacturer Specific,
            0x0183 => Self::BootSoftwareIdentification,
            0x0184 => Self::ApplicationSoftware,
            0x0185 => Self::ApplicationDataIdentification,
            0x0186 => Self::BootSoftwareFingerprint,
            0x0187 => Self::ApplicationSoftwareFingerprint,
            0x0188 => Self::ApplicationDataFingerprint,
            0x0189 => Self::ActiveDiagnosticSession,
            0x018A => Self::VehicleManufacturerSparePartNumber,
            0x018B => Self::VehicleManufacturerECUSoftwareNumber,
            0x018C => Self::VehicleManufacturerECUSoftwareVersionNumber,
            _ => return Err(Error::InvalidDiagnosticIdentifier(value)),
        })
    }
}

impl From<UDSIdentifier> for u16 {
    fn from(value: UDSIdentifier) -> Self {
        match value {
            UDSIdentifier::ISOSAEReserved(identifier) => identifier,
            UDSIdentifier::BootSoftwareIdentification => 0x0183,
            UDSIdentifier::ApplicationSoftware => 0x0184,
            UDSIdentifier::ApplicationDataIdentification => 0x0185,
            UDSIdentifier::BootSoftwareFingerprint => 0x0186,
            UDSIdentifier::ApplicationSoftwareFingerprint => 0x0187,
            UDSIdentifier::ApplicationDataFingerprint => 0x0188,
            UDSIdentifier::ActiveDiagnosticSession => 0x0189,
            UDSIdentifier::VehicleManufacturerSparePartNumber => 0x018A,
            UDSIdentifier::VehicleManufacturerECUSoftwareNumber => 0x018B,
            UDSIdentifier::VehicleManufacturerECUSoftwareVersionNumber => 0x018C,
        }
    }
}

/// Standard UDS Routine Identifier for the RoutineControl (0x31, 0x71) service
///
/// Some services will be defined by the Vehicle manufacturer or a system supplier,
/// and they must be implemented by the tester system.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UDSRoutineIdentifier {
    // 0x0000-0x00FF
    // 0xE300-0xEFFF
    // 0xFF02-0xFFFF
    ISOSAEReserved(u16),
    /// Represent Tachograph test result values
    ///
    /// // 0x0100-0x01FF
    TachographTestIds(u16),

    /// Vehicle Manufacturer Specific Routine Identifiers
    ///
    /// 0x0200-0xDFFF
    VehicleManufacturerSpecific(u16),

    /// 0xE000-0xE1FF
    OBDTestIds(u16),

    /// Execute Service Programming Loop (SPL)
    ///
    /// 0xE200
    ExecuteSPL,

    /// Deploy Loop Routine ID
    ///
    /// 0xE201
    DeployLoopRoutineID,

    /// 0xE202-0xE2FF
    SafetySystemRoutineID(u16),

    /// System Supplier Specific Routine Identifiers
    ///
    /// 0xF000-0xFEFF
    SystemSupplierSpecific(u16),

    /// Erase Memory
    ///
    /// 0xFF00
    EraseMemory,

    /// Check Programming Dependencies
    ///
    /// 0xFF01
    CheckProgrammingDependencies,
}

/// We know all values for the Routine Identifier, so we can implement From<u16> for UDSRoutineIdentifier
impl From<u16> for UDSRoutineIdentifier {
    fn from(value: u16) -> Self {
        match value {
            0x0000..=0x00FF | 0xE300..=0xEFFF | 0xFF02..=0xFFFF => Self::ISOSAEReserved(value),
            0x0100..=0x01FF => Self::TachographTestIds(value),
            0x0200..=0xDFFF => Self::VehicleManufacturerSpecific(value),
            0xE000..=0xE1FF => Self::OBDTestIds(value),
            0xE200 => Self::ExecuteSPL,
            0xE201 => Self::DeployLoopRoutineID,
            0xE202..=0xE2FF => Self::SafetySystemRoutineID(value),
            0xF000..=0xFEFF => Self::SystemSupplierSpecific(value),
            0xFF00 => Self::EraseMemory,
            0xFF01 => Self::CheckProgrammingDependencies,
        }
    }
}

impl From<UDSRoutineIdentifier> for u16 {
    fn from(value: UDSRoutineIdentifier) -> Self {
        match value {
            UDSRoutineIdentifier::ISOSAEReserved(identifier) => identifier,
            UDSRoutineIdentifier::TachographTestIds(identifier) => identifier,
            UDSRoutineIdentifier::VehicleManufacturerSpecific(identifier) => identifier,
            UDSRoutineIdentifier::OBDTestIds(identifier) => identifier,
            UDSRoutineIdentifier::ExecuteSPL => 0xE200,
            UDSRoutineIdentifier::DeployLoopRoutineID => 0xE201,
            UDSRoutineIdentifier::SafetySystemRoutineID(identifier) => identifier,
            UDSRoutineIdentifier::SystemSupplierSpecific(identifier) => identifier,
            UDSRoutineIdentifier::EraseMemory => 0xFF00,
            UDSRoutineIdentifier::CheckProgrammingDependencies => 0xFF01,
        }
    }
}
