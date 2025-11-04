//! DIDs are used to identify the data that is requested or sent in a diagnostic service.
use crate::{Error, Identifier, SingleValueWireFormat, traits::RoutineIdentifier};

/// C.1 DID - Diagnostic Data Identifier specified in ISO 14229-1
///
/// The identifiers listed here are defined and should be implemented by the vehicle manufacturer/system supplier.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum, clap::Parser))]
#[derive(Clone, Copy, Eq, Identifier, PartialEq)]
#[repr(u16)]
pub enum UDSIdentifier {
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u16),
    #[cfg_attr(feature = "clap", clap(skip))]
    VehicleManufacturerSpecific(u16),
    #[cfg_attr(feature = "clap", clap(skip))]
    SystemSupplierSpecific(u16),
    BootSoftwareIdentification = 0xF180,
    ApplicationSoftwareIdentification = 0xF181,
    ApplicationDataIdentification = 0xF182,
    BootSoftwareFingerprint = 0xF183,
    ApplicationSoftwareFingerprint = 0xF184,
    ApplicationDataFingerprint = 0xF185,
    ActiveDiagnosticSession = 0xF186,
    VehicleManufacturerSparePartNumber = 0xF187,
    VehicleManufacturerECUSoftwareNumber = 0xF188,
    VehicleManufacturerECUSoftwareVersionNumber = 0xF189,
    SystemSupplierIdentifier = 0xF18A,
    /// This value shall be used to reference the ECU (server) manufacturing date. Record data content and format shall be
    /// unsigned numeric, ASCII or BCD, and shall be ordered as Year, Month, Day.
    ECUManufacturingData = 0xF18B,
    /// Get the serial number of the ECU, format shall be server specific.
    ECUSerialNumber = 0xF18C,
    /// Request the supported functional units of the ECU.
    SupportedFunctionalUnits = 0xF18D,
    /// This value shall be used to reference the vehicle manufacturer order number for a kit (assembled parts bought as a whole for
    /// production e.g. cockpit), when the spare part number designates only the server (e.g. for aftersales). The record data content and
    /// format shall be server specific and defined by the vehicle manufacturer.
    VehicleManufacturerKitAssemblyPartNumber = 0xF18E,
    /// See 14229-1 C.1 for details on Regulation X information.
    /// Recurive ASCII string
    RegulationXSoftwareIdentificationNumbers = 0xF18F,
    VIN = 0xF190,
    VehicleManufacturerECUHardwareNumber = 0xF191,
    SystemSupplierECUHardwareNumber = 0xF192,
    SystemSupplierECUHardwareVersionNumber = 0xF193,
    SystemSupplierECUSoftwareNumber = 0xF194,
    SystemSupplierECUSoftwareVersionNumber = 0xF195,
    ExhaustRegulationOrTypeApprovalNumber = 0xF196,
    SystemNameOrEngineType = 0xF197,
    RepairShopOrTesterSerialNumber = 0xF198,
    /// When the server was last programmed, the record data content and format shall be
    /// unsigned numeric, ASCII or BCD, and shall be ordered as Year, Month, Day.
    ProgrammingDate = 0xF199,
    CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber = 0xF19A,
    CalibrationDate = 0xF19B,
    CalibrationEquipmentSoftwareNumber = 0xF19C,
    ECUInstallationDate = 0xF19D,
    ODXFile = 0xF19E,
    /// Used to reference the entity data identifier for a secured data transfer
    Entity = 0xF19F,
    UDSVersionData = 0xFF00,
    ReservedForISO15765_5 = 0xFF01,
}

impl TryFrom<u16> for UDSIdentifier {
    type Error = Error;

    #[allow(clippy::match_same_arms)]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0000..=0x00FF => Self::ISOSAEReserved(value),
            0x0100..=0xA5FF => Self::VehicleManufacturerSpecific(value),
            //0xA600..=0xA7FF => Self::ISOSAEReserved(value),
            0xA800..=0xACFF => Self::VehicleManufacturerSpecific(value),
            0xB000..=0xB1FF => Self::VehicleManufacturerSpecific(value),
            0xC000..=0xC2FF => Self::VehicleManufacturerSpecific(value),
            0xCF00..=0xEFFF => Self::VehicleManufacturerSpecific(value),
            // 0xF000..=0xF00F => Self::TractorTrailer(value),
            0xF010..=0xF0FF => Self::VehicleManufacturerSpecific(value),
            0xF100..=0xF17F => Self::VehicleManufacturerSpecific(value),
            // 0x0100..0xA5FF => Manufacturer Specific,
            0xF180 => Self::BootSoftwareIdentification,
            0xF181 => Self::ApplicationSoftwareIdentification,
            0xF182 => Self::ApplicationDataIdentification,
            0xF183 => Self::BootSoftwareFingerprint,
            0xF184 => Self::ApplicationSoftwareFingerprint,
            0xF185 => Self::ApplicationDataFingerprint,
            0xF186 => Self::ActiveDiagnosticSession,
            0xF187 => Self::VehicleManufacturerSparePartNumber,
            0xF188 => Self::VehicleManufacturerECUSoftwareNumber,
            0xF189 => Self::VehicleManufacturerECUSoftwareVersionNumber,
            0xF18A => Self::SystemSupplierIdentifier,
            0xF18B => Self::ECUManufacturingData,
            0xF18C => Self::ECUSerialNumber,
            0xF18D => Self::SupportedFunctionalUnits,
            0xF18E => Self::VehicleManufacturerKitAssemblyPartNumber,
            0xF18F => Self::RegulationXSoftwareIdentificationNumbers,
            0xF190 => Self::VIN,
            0xF191 => Self::VehicleManufacturerECUHardwareNumber,
            0xF192 => Self::SystemSupplierECUHardwareNumber,
            0xF193 => Self::SystemSupplierECUHardwareVersionNumber,
            0xF194 => Self::SystemSupplierECUSoftwareNumber,
            0xF195 => Self::SystemSupplierECUSoftwareVersionNumber,
            0xF196 => Self::ExhaustRegulationOrTypeApprovalNumber,
            0xF197 => Self::SystemNameOrEngineType,
            0xF198 => Self::RepairShopOrTesterSerialNumber,
            0xF199 => Self::ProgrammingDate,
            0xF19A => Self::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber,
            0xF19B => Self::CalibrationDate,
            0xF19C => Self::CalibrationEquipmentSoftwareNumber,
            0xF19D => Self::ECUInstallationDate,
            0xF19E => Self::ODXFile,
            0xF19F => Self::Entity,
            0xF1A0..=0xF1EF => Self::VehicleManufacturerSpecific(value),
            0xF1F0..=0xF1FF => Self::SystemSupplierSpecific(value),
            // 0xF200..=0xFDFF => Self::PeriodicDataIdentifier(value),
            // 0xF300..=0xF3FF => Self::DynamicallyDefined(value),
            // 0xF400..=0xF5FF => Self::OBD(value),
            // 0xF600..=0xF6FF => Self::OBDMonitor(value),
            // 0xF700..=0xF7FF => Self::OBD(value),
            // 0xF800..=0xF8FF => Self::OBDInfoType(value),
            // 0xF900..=0xF9FF => Self::Tachograph(value),
            // 0xFA00..=0xFA0F => Self::AirbagDeployment(value),
            0xFD00..=0xFEFF => Self::SystemSupplierSpecific(value),
            0xFF02..=0xFFFF => Self::ISOSAEReserved(value),

            _ => return Err(Error::InvalidDiagnosticIdentifier(value)),
        })
    }
}

impl From<UDSIdentifier> for u16 {
    #[allow(clippy::match_same_arms)]
    fn from(value: UDSIdentifier) -> Self {
        match value {
            UDSIdentifier::ISOSAEReserved(identifier) => identifier,
            UDSIdentifier::VehicleManufacturerSpecific(identifier) => identifier,
            UDSIdentifier::SystemSupplierSpecific(identifier) => identifier,
            UDSIdentifier::BootSoftwareIdentification => 0xF180,
            UDSIdentifier::ApplicationSoftwareIdentification => 0xF181,
            UDSIdentifier::ApplicationDataIdentification => 0xF182,
            UDSIdentifier::BootSoftwareFingerprint => 0xF183,
            UDSIdentifier::ApplicationSoftwareFingerprint => 0xF184,
            UDSIdentifier::ApplicationDataFingerprint => 0xF185,
            UDSIdentifier::ActiveDiagnosticSession => 0xF186,
            UDSIdentifier::VehicleManufacturerSparePartNumber => 0xF187,
            UDSIdentifier::VehicleManufacturerECUSoftwareNumber => 0xF188,
            UDSIdentifier::VehicleManufacturerECUSoftwareVersionNumber => 0xF189,
            UDSIdentifier::SystemSupplierIdentifier => 0xF18A,
            UDSIdentifier::ECUManufacturingData => 0xF18B,
            UDSIdentifier::ECUSerialNumber => 0xF18C,
            UDSIdentifier::SupportedFunctionalUnits => 0xF18D,
            UDSIdentifier::VehicleManufacturerKitAssemblyPartNumber => 0xF18E,
            UDSIdentifier::RegulationXSoftwareIdentificationNumbers => 0xF18F,
            UDSIdentifier::VIN => 0xF190,
            UDSIdentifier::VehicleManufacturerECUHardwareNumber => 0xF191,
            UDSIdentifier::SystemSupplierECUHardwareNumber => 0xF192,
            UDSIdentifier::SystemSupplierECUHardwareVersionNumber => 0xF193,
            UDSIdentifier::SystemSupplierECUSoftwareNumber => 0xF194,
            UDSIdentifier::SystemSupplierECUSoftwareVersionNumber => 0xF195,
            UDSIdentifier::ExhaustRegulationOrTypeApprovalNumber => 0xF196,
            UDSIdentifier::SystemNameOrEngineType => 0xF197,
            UDSIdentifier::RepairShopOrTesterSerialNumber => 0xF198,
            UDSIdentifier::ProgrammingDate => 0xF199,
            UDSIdentifier::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber => 0xF19A,
            UDSIdentifier::CalibrationDate => 0xF19B,
            UDSIdentifier::CalibrationEquipmentSoftwareNumber => 0xF19C,
            UDSIdentifier::ECUInstallationDate => 0xF19D,
            UDSIdentifier::ODXFile => 0xF19E,
            UDSIdentifier::Entity => 0xF19F,
            UDSIdentifier::UDSVersionData => 0xFF00,
            UDSIdentifier::ReservedForISO15765_5 => 0xFF01,
        }
    }
}

impl std::fmt::Display for UDSIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: u16 = (*self).into();
        write!(f, "{value:#06X?}")
    }
}

impl std::fmt::Debug for UDSIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: u16 = (*self).into();
        write!(f, "{value:#06X}")
    }
}

/// Standard UDS Routine Identifier for the `RoutineControl` (0x31, 0x71) service
///
/// Some services will be defined by the Vehicle manufacturer or a system supplier,
/// and they must be implemented by the tester system.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, Identifier, PartialEq)]
#[repr(u16)]
pub enum UDSRoutineIdentifier {
    // 0x0000-0x00FF
    // 0xE300-0xEFFF
    // 0xFF02-0xFFFF
    ISOSAEReserved(u16),
    /// Represent Tachograph test result values
    ///
    /// 0x0100-0x01FF
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
    ExecuteSPL = 0xE200,

    /// Deploy Loop Routine ID
    ///
    /// 0xE201
    DeployLoopRoutineID = 0xE201,

    /// 0xE202-0xE2FF
    SafetySystemRoutineID(u16),

    /// System Supplier Specific Routine Identifiers
    ///
    /// 0xF000-0xFEFF
    SystemSupplierSpecific(u16),

    /// Erase Memory
    ///
    /// 0xFF00
    EraseMemory = 0xFF00,

    /// Check Programming Dependencies
    ///
    /// 0xFF01
    CheckProgrammingDependencies = 0xFF01,
}

/// We know all values for the Routine Identifier, so we can implement `From<u16>` for `UDSRoutineIdentifier`
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
    #[allow(clippy::match_same_arms)]
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

impl SingleValueWireFormat for UDSRoutineIdentifier {}
impl RoutineIdentifier for UDSRoutineIdentifier {}
