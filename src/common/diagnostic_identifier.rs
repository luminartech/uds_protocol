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
    BootSoftwareIdentification = Self::BOOT_SOFTWARE_IDENTIFICATION,
    ApplicationSoftwareIdentification = Self::APPLICATION_SOFTWARE_IDENTIFICATION,
    ApplicationDataIdentification = Self::APPLICATION_DATA_IDENTIFICATION,
    BootSoftwareFingerprint = Self::BOOT_SOFTWARE_FINGERPRINT,
    ApplicationSoftwareFingerprint = Self::APPLICATION_SOFTWARE_FINGERPRINT,
    ApplicationDataFingerprint = Self::APPLICATION_DATA_FINGERPRINT,
    ActiveDiagnosticSession = Self::ACTIVE_DIAGNOSTIC_SESSION,
    VehicleManufacturerSparePartNumber = Self::VEHICLE_MANUFACTURER_SPARE_PART_NUMBER,
    VehicleManufacturerECUSoftwareNumber = Self::VEHICLE_MANUFACTURER_ECU_SOFTWARE_NUMBER,
    VehicleManufacturerECUSoftwareVersionNumber =
        Self::VEHICLE_MANUFACTURER_ECU_SOFTWARE_VERSION_NUMBER,
    SystemSupplierIdentifier = Self::SYSTEM_SUPPLIER_IDENTIFIER,
    /// This value shall be used to reference the ECU (server) manufacturing date. Record data content and format shall be
    /// unsigned numeric, ASCII or BCD, and shall be ordered as Year, Month, Day.
    ECUManufacturingData = Self::ECU_MANUFACTURING_DATA,
    /// Get the serial number of the ECU, format shall be server specific.
    ECUSerialNumber = Self::ECU_SERIAL_NUMBER,
    /// Request the supported functional units of the ECU.
    SupportedFunctionalUnits = Self::SUPPORTED_FUNCTIONAL_UNITS,
    /// This value shall be used to reference the vehicle manufacturer order number for a kit (assembled parts bought as a whole for
    /// production e.g. cockpit), when the spare part number designates only the server (e.g. for aftersales). The record data content and
    /// format shall be server specific and defined by the vehicle manufacturer.
    VehicleManufacturerKitAssemblyPartNumber = Self::VEHICLE_MANUFACTURER_KIT_ASSEMBLY_PART_NUMBER,
    /// See 14229-1 C.1 for details on Regulation X information.
    /// Recurive ASCII string
    RegulationXSoftwareIdentificationNumbers = Self::REGULATION_X_SOFTWARE_IDENTIFICATION_NUMBERS,
    VehicleIdentificationNumber = Self::VEHICLE_IDENTIFICATION_NUMBER,
    VehicleManufacturerECUHardwareNumber = Self::VEHICLE_MANUFACTURER_ECU_HARDWARE_NUMBER,
    SystemSupplierECUHardwareNumber = Self::SYSTEM_SUPPLIER_ECU_HARDWARE_NUMBER,
    SystemSupplierECUHardwareVersionNumber = Self::SYSTEM_SUPPLIER_ECU_HARDWARE_VERSION_NUMBER,
    SystemSupplierECUSoftwareNumber = Self::SYSTEM_SUPPLIER_ECU_SOFTWARE_NUMBER,
    SystemSupplierECUSoftwareVersionNumber = Self::SYSTEM_SUPPLIER_ECU_SOFTWARE_VERSION_NUMBER,
    ExhaustRegulationOrTypeApprovalNumber = Self::EXHAUST_REGULATION_OR_TYPE_APPROVAL_NUMBER,
    SystemNameOrEngineType = Self::SYSTEM_NAME_OR_ENGINE_TYPE,
    RepairShopOrTesterSerialNumber = Self::REPAIR_SHOP_OR_TESTER_SERIAL_NUMBER,
    /// When the server was last programmed, the record data content and format shall be
    /// unsigned numeric, ASCII or BCD, and shall be ordered as Year, Month, Day.
    ProgrammingDate = Self::PROGRAMMING_DATE,
    CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber =
        Self::CALIBRATION_REPAIR_SHOP_CODE_OR_CALIBRATION_EQUIPMENT_SERIAL_NUMBER,
    CalibrationDate = Self::CALIBRATION_DATE,
    CalibrationEquipmentSoftwareNumber = Self::CALIBRATION_EQUIPMENT_SOFTWARE_NUMBER,
    ECUInstallationDate = Self::ECU_INSTALLATION_DATE,
    ODXFile = Self::ODX_FILE,
    /// Used to reference the entity data identifier for a secured data transfer
    Entity = Self::ENTITY,
    UDSVersionData = Self::UDS_VERSION_DATA,
    ReservedForISO15765_5 = Self::RESERVED_FOR_ISO15765_5,
}

impl UDSIdentifier {
    pub const ISO_SAE_RESERVED_START: u16 = 0x0000;
    pub const ISO_SAE_RESERVED_END: u16 = 0x00FF;
    pub const VEHICLE_MANUFACTURER_SPECIFIC_START: u16 = 0xF100;
    pub const VEHICLE_MANUFACTURER_SPECIFIC_END: u16 = 0xF17F;
    pub const BOOT_SOFTWARE_IDENTIFICATION: u16 = 0xF180;
    pub const APPLICATION_SOFTWARE_IDENTIFICATION: u16 = 0xF181;
    pub const APPLICATION_DATA_IDENTIFICATION: u16 = 0xF182;
    pub const BOOT_SOFTWARE_FINGERPRINT: u16 = 0xF183;
    pub const APPLICATION_SOFTWARE_FINGERPRINT: u16 = 0xF184;
    pub const APPLICATION_DATA_FINGERPRINT: u16 = 0xF185;
    pub const ACTIVE_DIAGNOSTIC_SESSION: u16 = 0xF186;
    pub const VEHICLE_MANUFACTURER_SPARE_PART_NUMBER: u16 = 0xF187;
    pub const VEHICLE_MANUFACTURER_ECU_SOFTWARE_NUMBER: u16 = 0xF188;
    pub const VEHICLE_MANUFACTURER_ECU_SOFTWARE_VERSION_NUMBER: u16 = 0xF189;
    pub const SYSTEM_SUPPLIER_IDENTIFIER: u16 = 0xF18A;
    pub const ECU_MANUFACTURING_DATA: u16 = 0xF18B;
    pub const ECU_SERIAL_NUMBER: u16 = 0xF18C;
    pub const SUPPORTED_FUNCTIONAL_UNITS: u16 = 0xF18D;
    pub const VEHICLE_MANUFACTURER_KIT_ASSEMBLY_PART_NUMBER: u16 = 0xF18E;
    pub const REGULATION_X_SOFTWARE_IDENTIFICATION_NUMBERS: u16 = 0xF18F;
    pub const VEHICLE_IDENTIFICATION_NUMBER: u16 = 0xF190;
    pub const VEHICLE_MANUFACTURER_ECU_HARDWARE_NUMBER: u16 = 0xF191;
    pub const SYSTEM_SUPPLIER_ECU_HARDWARE_NUMBER: u16 = 0xF192;
    pub const SYSTEM_SUPPLIER_ECU_HARDWARE_VERSION_NUMBER: u16 = 0xF193;
    pub const SYSTEM_SUPPLIER_ECU_SOFTWARE_NUMBER: u16 = 0xF194;
    pub const SYSTEM_SUPPLIER_ECU_SOFTWARE_VERSION_NUMBER: u16 = 0xF195;
    pub const EXHAUST_REGULATION_OR_TYPE_APPROVAL_NUMBER: u16 = 0xF196;
    pub const SYSTEM_NAME_OR_ENGINE_TYPE: u16 = 0xF197;
    pub const REPAIR_SHOP_OR_TESTER_SERIAL_NUMBER: u16 = 0xF198;
    pub const PROGRAMMING_DATE: u16 = 0xF199;
    pub const CALIBRATION_REPAIR_SHOP_CODE_OR_CALIBRATION_EQUIPMENT_SERIAL_NUMBER: u16 = 0xF19A;
    pub const CALIBRATION_DATE: u16 = 0xF19B;
    pub const CALIBRATION_EQUIPMENT_SOFTWARE_NUMBER: u16 = 0xF19C;
    pub const ECU_INSTALLATION_DATE: u16 = 0xF19D;
    pub const ODX_FILE: u16 = 0xF19E;
    pub const ENTITY: u16 = 0xF19F;
    pub const UDS_VERSION_DATA: u16 = 0xFF00;
    pub const RESERVED_FOR_ISO15765_5: u16 = 0xFF01;
    pub const SYSTEM_SUPPLIER_SPECIFIC_START: u16 = 0xFD00;
    pub const SYSTEM_SUPPLIER_SPECIFIC_END: u16 = 0xFEFF;
    pub const ISO_RESERVED_START: u16 = 0xFF02;
    pub const ISO_RESERVED_END: u16 = 0xFFFF;
}

impl TryFrom<u16> for UDSIdentifier {
    type Error = Error;

    #[allow(clippy::match_same_arms)]
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            UDSIdentifier::ISO_SAE_RESERVED_START..=UDSIdentifier::ISO_SAE_RESERVED_END => {
                UDSIdentifier::ISOSAEReserved(value)
            }
            UDSIdentifier::VEHICLE_MANUFACTURER_SPECIFIC_START
                ..=UDSIdentifier::VEHICLE_MANUFACTURER_SPECIFIC_END => {
                UDSIdentifier::VehicleManufacturerSpecific(value)
            }
            // 0x0100..0xA5FF => Manufacturer Specific,
            UDSIdentifier::BOOT_SOFTWARE_IDENTIFICATION => {
                UDSIdentifier::BootSoftwareIdentification
            }
            UDSIdentifier::APPLICATION_SOFTWARE_IDENTIFICATION => {
                UDSIdentifier::ApplicationSoftwareIdentification
            }
            UDSIdentifier::APPLICATION_DATA_IDENTIFICATION => {
                UDSIdentifier::ApplicationDataIdentification
            }
            UDSIdentifier::BOOT_SOFTWARE_FINGERPRINT => UDSIdentifier::BootSoftwareFingerprint,
            UDSIdentifier::APPLICATION_SOFTWARE_FINGERPRINT => {
                UDSIdentifier::ApplicationSoftwareFingerprint
            }
            UDSIdentifier::APPLICATION_DATA_FINGERPRINT => {
                UDSIdentifier::ApplicationDataFingerprint
            }
            UDSIdentifier::ACTIVE_DIAGNOSTIC_SESSION => UDSIdentifier::ActiveDiagnosticSession,
            UDSIdentifier::VEHICLE_MANUFACTURER_SPARE_PART_NUMBER => {
                UDSIdentifier::VehicleManufacturerSparePartNumber
            }
            UDSIdentifier::VEHICLE_MANUFACTURER_ECU_SOFTWARE_NUMBER => {
                UDSIdentifier::VehicleManufacturerECUSoftwareNumber
            }
            UDSIdentifier::VEHICLE_MANUFACTURER_ECU_SOFTWARE_VERSION_NUMBER => {
                UDSIdentifier::VehicleManufacturerECUSoftwareVersionNumber
            }
            UDSIdentifier::SYSTEM_SUPPLIER_IDENTIFIER => UDSIdentifier::SystemSupplierIdentifier,
            UDSIdentifier::ECU_MANUFACTURING_DATA => UDSIdentifier::ECUManufacturingData,
            UDSIdentifier::ECU_SERIAL_NUMBER => UDSIdentifier::ECUSerialNumber,
            UDSIdentifier::SUPPORTED_FUNCTIONAL_UNITS => UDSIdentifier::SupportedFunctionalUnits,
            UDSIdentifier::VEHICLE_MANUFACTURER_KIT_ASSEMBLY_PART_NUMBER => {
                UDSIdentifier::VehicleManufacturerKitAssemblyPartNumber
            }
            UDSIdentifier::REGULATION_X_SOFTWARE_IDENTIFICATION_NUMBERS => {
                UDSIdentifier::RegulationXSoftwareIdentificationNumbers
            }
            UDSIdentifier::VEHICLE_IDENTIFICATION_NUMBER => {
                UDSIdentifier::VehicleIdentificationNumber
            }
            UDSIdentifier::VEHICLE_MANUFACTURER_ECU_HARDWARE_NUMBER => {
                UDSIdentifier::VehicleManufacturerECUHardwareNumber
            }
            UDSIdentifier::SYSTEM_SUPPLIER_ECU_HARDWARE_NUMBER => {
                UDSIdentifier::SystemSupplierECUHardwareNumber
            }
            UDSIdentifier::SYSTEM_SUPPLIER_ECU_HARDWARE_VERSION_NUMBER => {
                UDSIdentifier::SystemSupplierECUHardwareVersionNumber
            }
            UDSIdentifier::SYSTEM_SUPPLIER_ECU_SOFTWARE_NUMBER => {
                UDSIdentifier::SystemSupplierECUSoftwareNumber
            }
            UDSIdentifier::SYSTEM_SUPPLIER_ECU_SOFTWARE_VERSION_NUMBER => {
                UDSIdentifier::SystemSupplierECUSoftwareVersionNumber
            }
            UDSIdentifier::EXHAUST_REGULATION_OR_TYPE_APPROVAL_NUMBER => {
                UDSIdentifier::ExhaustRegulationOrTypeApprovalNumber
            }
            UDSIdentifier::SYSTEM_NAME_OR_ENGINE_TYPE => UDSIdentifier::SystemNameOrEngineType,
            UDSIdentifier::REPAIR_SHOP_OR_TESTER_SERIAL_NUMBER => {
                UDSIdentifier::RepairShopOrTesterSerialNumber
            }
            UDSIdentifier::PROGRAMMING_DATE => UDSIdentifier::ProgrammingDate,
            UDSIdentifier::CALIBRATION_REPAIR_SHOP_CODE_OR_CALIBRATION_EQUIPMENT_SERIAL_NUMBER => {
                UDSIdentifier::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber
            }
            UDSIdentifier::CALIBRATION_DATE => UDSIdentifier::CalibrationDate,
            UDSIdentifier::CALIBRATION_EQUIPMENT_SOFTWARE_NUMBER => {
                UDSIdentifier::CalibrationEquipmentSoftwareNumber
            }
            UDSIdentifier::ECU_INSTALLATION_DATE => UDSIdentifier::ECUInstallationDate,
            UDSIdentifier::ODX_FILE => UDSIdentifier::ODXFile,
            UDSIdentifier::ENTITY => UDSIdentifier::Entity,
            UDSIdentifier::SYSTEM_SUPPLIER_SPECIFIC_START
                ..=UDSIdentifier::SYSTEM_SUPPLIER_SPECIFIC_END => {
                UDSIdentifier::SystemSupplierSpecific(value)
            }
            UDSIdentifier::ISO_RESERVED_START..=UDSIdentifier::ISO_RESERVED_END => {
                UDSIdentifier::ISOSAEReserved(value)
            }

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
            UDSIdentifier::BootSoftwareIdentification => {
                UDSIdentifier::BOOT_SOFTWARE_IDENTIFICATION
            }
            UDSIdentifier::ApplicationSoftwareIdentification => {
                UDSIdentifier::APPLICATION_SOFTWARE_IDENTIFICATION
            }
            UDSIdentifier::ApplicationDataIdentification => {
                UDSIdentifier::APPLICATION_DATA_IDENTIFICATION
            }
            UDSIdentifier::BootSoftwareFingerprint => UDSIdentifier::BOOT_SOFTWARE_FINGERPRINT,
            UDSIdentifier::ApplicationSoftwareFingerprint => {
                UDSIdentifier::APPLICATION_SOFTWARE_FINGERPRINT
            }
            UDSIdentifier::ApplicationDataFingerprint => {
                UDSIdentifier::APPLICATION_DATA_FINGERPRINT
            }
            UDSIdentifier::ActiveDiagnosticSession => UDSIdentifier::ACTIVE_DIAGNOSTIC_SESSION,
            UDSIdentifier::VehicleManufacturerSparePartNumber => {
                UDSIdentifier::VEHICLE_MANUFACTURER_SPARE_PART_NUMBER
            }
            UDSIdentifier::VehicleManufacturerECUSoftwareNumber => {
                UDSIdentifier::VEHICLE_MANUFACTURER_ECU_SOFTWARE_NUMBER
            }
            UDSIdentifier::VehicleManufacturerECUSoftwareVersionNumber => {
                UDSIdentifier::VEHICLE_MANUFACTURER_ECU_SOFTWARE_VERSION_NUMBER
            }
            UDSIdentifier::SystemSupplierIdentifier => UDSIdentifier::SYSTEM_SUPPLIER_IDENTIFIER,
            UDSIdentifier::ECUManufacturingData => UDSIdentifier::ECU_MANUFACTURING_DATA,
            UDSIdentifier::ECUSerialNumber => UDSIdentifier::ECU_SERIAL_NUMBER,
            UDSIdentifier::SupportedFunctionalUnits => UDSIdentifier::SUPPORTED_FUNCTIONAL_UNITS,
            UDSIdentifier::VehicleManufacturerKitAssemblyPartNumber => {
                UDSIdentifier::VEHICLE_MANUFACTURER_KIT_ASSEMBLY_PART_NUMBER
            }
            UDSIdentifier::RegulationXSoftwareIdentificationNumbers => {
                UDSIdentifier::REGULATION_X_SOFTWARE_IDENTIFICATION_NUMBERS
            }
            UDSIdentifier::VehicleIdentificationNumber => {
                UDSIdentifier::VEHICLE_IDENTIFICATION_NUMBER
            }
            UDSIdentifier::VehicleManufacturerECUHardwareNumber => {
                UDSIdentifier::VEHICLE_MANUFACTURER_ECU_HARDWARE_NUMBER
            }
            UDSIdentifier::SystemSupplierECUHardwareNumber => {
                UDSIdentifier::SYSTEM_SUPPLIER_ECU_HARDWARE_NUMBER
            }
            UDSIdentifier::SystemSupplierECUHardwareVersionNumber => {
                UDSIdentifier::SYSTEM_SUPPLIER_ECU_HARDWARE_VERSION_NUMBER
            }
            UDSIdentifier::SystemSupplierECUSoftwareNumber => {
                UDSIdentifier::SYSTEM_SUPPLIER_ECU_SOFTWARE_NUMBER
            }
            UDSIdentifier::SystemSupplierECUSoftwareVersionNumber => {
                UDSIdentifier::SYSTEM_SUPPLIER_ECU_SOFTWARE_VERSION_NUMBER
            }
            UDSIdentifier::ExhaustRegulationOrTypeApprovalNumber => {
                UDSIdentifier::EXHAUST_REGULATION_OR_TYPE_APPROVAL_NUMBER
            }
            UDSIdentifier::SystemNameOrEngineType => UDSIdentifier::SYSTEM_NAME_OR_ENGINE_TYPE,
            UDSIdentifier::RepairShopOrTesterSerialNumber => {
                UDSIdentifier::REPAIR_SHOP_OR_TESTER_SERIAL_NUMBER
            }
            UDSIdentifier::ProgrammingDate => UDSIdentifier::PROGRAMMING_DATE,
            UDSIdentifier::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber => {
                UDSIdentifier::CALIBRATION_REPAIR_SHOP_CODE_OR_CALIBRATION_EQUIPMENT_SERIAL_NUMBER
            }
            UDSIdentifier::CalibrationDate => UDSIdentifier::CALIBRATION_DATE,
            UDSIdentifier::CalibrationEquipmentSoftwareNumber => {
                UDSIdentifier::CALIBRATION_EQUIPMENT_SOFTWARE_NUMBER
            }
            UDSIdentifier::ECUInstallationDate => UDSIdentifier::ECU_INSTALLATION_DATE,
            UDSIdentifier::ODXFile => UDSIdentifier::ODX_FILE,
            UDSIdentifier::Entity => UDSIdentifier::ENTITY,
            UDSIdentifier::UDSVersionData => UDSIdentifier::UDS_VERSION_DATA,
            UDSIdentifier::ReservedForISO15765_5 => UDSIdentifier::RESERVED_FOR_ISO15765_5,
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
