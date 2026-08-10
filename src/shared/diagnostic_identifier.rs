//! DIDs are used to identify the data that is requested or sent in a diagnostic service.

/// C.1 DID - Diagnostic Data Identifier specified in ISO 14229-1
///
/// The identifiers listed here are defined and should be implemented by the vehicle manufacturer/system supplier.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum UdsIdentifier {
    /// DID reserved by ISO/SAE (ranges `0x0000–0x00FF`, `0xFF02–0xFFFF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    IsoSaeReserved(u16),
    /// Vehicle-manufacturer–specific DID (multiple ranges, see ISO 14229-1 Table C.1).
    #[cfg_attr(feature = "clap", clap(skip))]
    VehicleManufacturerSpecific(u16),
    /// System-supplier–specific DID (`0xFD00–0xFEFF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    SystemSupplierSpecific(u16),
    /// Reserved for legislative use (multiple ranges per ISO 14229-1 Table C.1).
    #[cfg_attr(feature = "clap", clap(skip))]
    ReservedForLegislativeUse(u16),
    /// Network configuration data for tractor-trailer application (`0xF000–0xF00F`).
    #[cfg_attr(feature = "clap", clap(skip))]
    NetworkConfigDataForTractorTrailer(u16),
    /// Identification option vehicle-manufacturer–specific (`0xF100–0xF17F`, `0xF1A0–0xF1EF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    IdentificationOptionVehicleManufacturerSpecific(u16),
    /// Identification option system-supplier–specific (`0xF1F0–0xF1FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    IdentificationOptionSystemSupplierSpecific(u16),
    /// Periodic data identifier (`0xF200–0xF2FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    PeriodicDataIdentifier(u16),
    /// Dynamically defined data identifier (`0xF300–0xF3FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    DynamicallyDefineDataIdentifier(u16),
    /// OBD data identifier (`0xF400–0xF5FF`, `0xF700–0xF7FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    ObdDataIdentifier(u16),
    /// OBD monitor data identifier (`0xF600–0xF6FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    ObdMonitorDataIdentifier(u16),
    /// OBD info-type data identifier (`0xF800–0xF8FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    ObdInfoTypeDataIdentifier(u16),
    /// Tachograph data identifier (`0xF900–0xF9FF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    TachographDataIdentifier(u16),
    /// Airbag deployment data identifier (`0xFA00–0xFA0F`).
    #[cfg_attr(feature = "clap", clap(skip))]
    AirbagDeploymentDataIdentifier(u16),
    /// Number of EDR devices (`0xFA10`).
    NumberOfEdrDevices,
    /// EDR identification (`0xFA11`).
    EdrIdentification,
    /// EDR device address information (`0xFA12`).
    EdrDeviceAddressInformation,
    /// EDR entries (`0xFA13–0xFA18`).
    #[cfg_attr(feature = "clap", clap(skip))]
    EdrEntries(u16),
    /// Safety system data identifier (`0xFA19–0xFAFF`).
    #[cfg_attr(feature = "clap", clap(skip))]
    SafetySystemDataIdentifier(u16),
    /// Boot software identification (`0xF180`).
    BootSoftwareIdentification,
    /// Application software identification (`0xF181`).
    ApplicationSoftwareIdentification,
    /// Application data identification (`0xF182`).
    ApplicationDataIdentification,
    /// Boot software fingerprint (`0xF183`).
    BootSoftwareFingerprint,
    /// Application software fingerprint (`0xF184`).
    ApplicationSoftwareFingerprint,
    /// Application data fingerprint (`0xF185`).
    ApplicationDataFingerprint,
    /// Active diagnostic session (`0xF186`).
    ActiveDiagnosticSession,
    /// Vehicle manufacturer spare-part number (`0xF187`).
    VehicleManufacturerSparePartNumber,
    /// Vehicle manufacturer ECU software number (`0xF188`).
    VehicleManufacturerEcuSoftwareNumber,
    /// Vehicle manufacturer ECU software version number (`0xF189`).
    VehicleManufacturerEcuSoftwareVersionNumber,
    /// System supplier identifier (`0xF18A`).
    SystemSupplierIdentifier,
    /// This value shall be used to reference the ECU (server) manufacturing date. Record data content and format shall be
    /// unsigned numeric, ASCII or BCD, and shall be ordered as Year, Month, Day.
    EcuManufacturingData,
    /// Get the serial number of the ECU, format shall be server specific.
    EcuSerialNumber,
    /// Request the supported functional units of the ECU.
    SupportedFunctionalUnits,
    /// This value shall be used to reference the vehicle manufacturer order number for a kit (assembled parts bought as a whole for
    /// production e.g. cockpit), when the spare part number designates only the server (e.g. for aftersales). The record data content and
    /// format shall be server specific and defined by the vehicle manufacturer.
    VehicleManufacturerKitAssemblyPartNumber,
    /// See 14229-1 C.1 for details on Regulation X information.
    /// Recursive ASCII string
    RegulationXSoftwareIdentificationNumbers,
    /// Vehicle Identification Number (`0xF190`).
    Vin,
    /// Vehicle manufacturer ECU hardware number (`0xF191`).
    VehicleManufacturerEcuHardwareNumber,
    /// System supplier ECU hardware number (`0xF192`).
    SystemSupplierEcuHardwareNumber,
    /// System supplier ECU hardware version number (`0xF193`).
    SystemSupplierEcuHardwareVersionNumber,
    /// System supplier ECU software number (`0xF194`).
    SystemSupplierEcuSoftwareNumber,
    /// System supplier ECU software version number (`0xF195`).
    SystemSupplierEcuSoftwareVersionNumber,
    /// Exhaust regulation or type approval number (`0xF196`).
    ExhaustRegulationOrTypeApprovalNumber,
    /// System name or engine type (`0xF197`).
    SystemNameOrEngineType,
    /// Repair shop or tester serial number (`0xF198`).
    RepairShopOrTesterSerialNumber,
    /// When the server was last programmed, the record data content and format shall be
    /// unsigned numeric, ASCII or BCD, and shall be ordered as Year, Month, Day.
    ProgrammingDate,
    /// Calibration repair-shop code or calibration equipment serial number (`0xF19A`).
    CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber,
    /// Calibration date (`0xF19B`).
    CalibrationDate,
    /// Calibration equipment software number (`0xF19C`).
    CalibrationEquipmentSoftwareNumber,
    /// ECU installation date (`0xF19D`).
    EcuInstallationDate,
    /// ODX file identifier (`0xF19E`).
    OdxFile,
    /// Used to reference the entity data identifier for a secured data transfer
    Entity,
    /// UDS version data (`0xFF00`).
    UdsVersionData,
    /// Reserved for ISO 15765-5 (`0xFF01`).
    ReservedForIso15765_5,
}

impl From<u16> for UdsIdentifier {
    #[allow(clippy::match_same_arms)]
    fn from(value: u16) -> Self {
        match value {
            0x0000..=0x00FF => Self::IsoSaeReserved(value),
            0x0100..=0xA5FF => Self::VehicleManufacturerSpecific(value),
            0xA600..=0xA7FF => Self::ReservedForLegislativeUse(value),
            0xA800..=0xACFF => Self::VehicleManufacturerSpecific(value),
            0xAD00..=0xAFFF => Self::ReservedForLegislativeUse(value),
            0xB000..=0xB1FF => Self::VehicleManufacturerSpecific(value),
            0xB200..=0xBFFF => Self::ReservedForLegislativeUse(value),
            0xC000..=0xC2FF => Self::VehicleManufacturerSpecific(value),
            0xC300..=0xCEFF => Self::ReservedForLegislativeUse(value),
            0xCF00..=0xEFFF => Self::VehicleManufacturerSpecific(value),
            0xF000..=0xF00F => Self::NetworkConfigDataForTractorTrailer(value),
            0xF010..=0xF0FF => Self::VehicleManufacturerSpecific(value),
            0xF100..=0xF17F => Self::IdentificationOptionVehicleManufacturerSpecific(value),
            0xF180 => Self::BootSoftwareIdentification,
            0xF181 => Self::ApplicationSoftwareIdentification,
            0xF182 => Self::ApplicationDataIdentification,
            0xF183 => Self::BootSoftwareFingerprint,
            0xF184 => Self::ApplicationSoftwareFingerprint,
            0xF185 => Self::ApplicationDataFingerprint,
            0xF186 => Self::ActiveDiagnosticSession,
            0xF187 => Self::VehicleManufacturerSparePartNumber,
            0xF188 => Self::VehicleManufacturerEcuSoftwareNumber,
            0xF189 => Self::VehicleManufacturerEcuSoftwareVersionNumber,
            0xF18A => Self::SystemSupplierIdentifier,
            0xF18B => Self::EcuManufacturingData,
            0xF18C => Self::EcuSerialNumber,
            0xF18D => Self::SupportedFunctionalUnits,
            0xF18E => Self::VehicleManufacturerKitAssemblyPartNumber,
            0xF18F => Self::RegulationXSoftwareIdentificationNumbers,
            0xF190 => Self::Vin,
            0xF191 => Self::VehicleManufacturerEcuHardwareNumber,
            0xF192 => Self::SystemSupplierEcuHardwareNumber,
            0xF193 => Self::SystemSupplierEcuHardwareVersionNumber,
            0xF194 => Self::SystemSupplierEcuSoftwareNumber,
            0xF195 => Self::SystemSupplierEcuSoftwareVersionNumber,
            0xF196 => Self::ExhaustRegulationOrTypeApprovalNumber,
            0xF197 => Self::SystemNameOrEngineType,
            0xF198 => Self::RepairShopOrTesterSerialNumber,
            0xF199 => Self::ProgrammingDate,
            0xF19A => Self::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber,
            0xF19B => Self::CalibrationDate,
            0xF19C => Self::CalibrationEquipmentSoftwareNumber,
            0xF19D => Self::EcuInstallationDate,
            0xF19E => Self::OdxFile,
            0xF19F => Self::Entity,
            0xF1A0..=0xF1EF => Self::IdentificationOptionVehicleManufacturerSpecific(value),
            0xF1F0..=0xF1FF => Self::IdentificationOptionSystemSupplierSpecific(value),
            0xF200..=0xF2FF => Self::PeriodicDataIdentifier(value),
            0xF300..=0xF3FF => Self::DynamicallyDefineDataIdentifier(value),
            0xF400..=0xF5FF => Self::ObdDataIdentifier(value),
            0xF600..=0xF6FF => Self::ObdMonitorDataIdentifier(value),
            0xF700..=0xF7FF => Self::ObdDataIdentifier(value),
            0xF800..=0xF8FF => Self::ObdInfoTypeDataIdentifier(value),
            0xF900..=0xF9FF => Self::TachographDataIdentifier(value),
            0xFA00..=0xFA0F => Self::AirbagDeploymentDataIdentifier(value),
            0xFA10 => Self::NumberOfEdrDevices,
            0xFA11 => Self::EdrIdentification,
            0xFA12 => Self::EdrDeviceAddressInformation,
            0xFA13..=0xFA18 => Self::EdrEntries(value),
            0xFA19..=0xFAFF => Self::SafetySystemDataIdentifier(value),
            0xFB00..=0xFCFF => Self::ReservedForLegislativeUse(value),
            0xFD00..=0xFEFF => Self::SystemSupplierSpecific(value),
            0xFF00 => Self::UdsVersionData,
            0xFF01 => Self::ReservedForIso15765_5,
            0xFF02..=0xFFFF => Self::IsoSaeReserved(value),
        }
    }
}

impl From<UdsIdentifier> for u16 {
    #[allow(clippy::match_same_arms)]
    fn from(value: UdsIdentifier) -> Self {
        match value {
            UdsIdentifier::IsoSaeReserved(v) => v,
            UdsIdentifier::VehicleManufacturerSpecific(v) => v,
            UdsIdentifier::SystemSupplierSpecific(v) => v,
            UdsIdentifier::ReservedForLegislativeUse(v) => v,
            UdsIdentifier::NetworkConfigDataForTractorTrailer(v) => v,
            UdsIdentifier::IdentificationOptionVehicleManufacturerSpecific(v) => v,
            UdsIdentifier::IdentificationOptionSystemSupplierSpecific(v) => v,
            UdsIdentifier::PeriodicDataIdentifier(v) => v,
            UdsIdentifier::DynamicallyDefineDataIdentifier(v) => v,
            UdsIdentifier::ObdDataIdentifier(v) => v,
            UdsIdentifier::ObdMonitorDataIdentifier(v) => v,
            UdsIdentifier::ObdInfoTypeDataIdentifier(v) => v,
            UdsIdentifier::TachographDataIdentifier(v) => v,
            UdsIdentifier::AirbagDeploymentDataIdentifier(v) => v,
            UdsIdentifier::NumberOfEdrDevices => 0xFA10,
            UdsIdentifier::EdrIdentification => 0xFA11,
            UdsIdentifier::EdrDeviceAddressInformation => 0xFA12,
            UdsIdentifier::EdrEntries(v) => v,
            UdsIdentifier::SafetySystemDataIdentifier(v) => v,
            UdsIdentifier::BootSoftwareIdentification => 0xF180,
            UdsIdentifier::ApplicationSoftwareIdentification => 0xF181,
            UdsIdentifier::ApplicationDataIdentification => 0xF182,
            UdsIdentifier::BootSoftwareFingerprint => 0xF183,
            UdsIdentifier::ApplicationSoftwareFingerprint => 0xF184,
            UdsIdentifier::ApplicationDataFingerprint => 0xF185,
            UdsIdentifier::ActiveDiagnosticSession => 0xF186,
            UdsIdentifier::VehicleManufacturerSparePartNumber => 0xF187,
            UdsIdentifier::VehicleManufacturerEcuSoftwareNumber => 0xF188,
            UdsIdentifier::VehicleManufacturerEcuSoftwareVersionNumber => 0xF189,
            UdsIdentifier::SystemSupplierIdentifier => 0xF18A,
            UdsIdentifier::EcuManufacturingData => 0xF18B,
            UdsIdentifier::EcuSerialNumber => 0xF18C,
            UdsIdentifier::SupportedFunctionalUnits => 0xF18D,
            UdsIdentifier::VehicleManufacturerKitAssemblyPartNumber => 0xF18E,
            UdsIdentifier::RegulationXSoftwareIdentificationNumbers => 0xF18F,
            UdsIdentifier::Vin => 0xF190,
            UdsIdentifier::VehicleManufacturerEcuHardwareNumber => 0xF191,
            UdsIdentifier::SystemSupplierEcuHardwareNumber => 0xF192,
            UdsIdentifier::SystemSupplierEcuHardwareVersionNumber => 0xF193,
            UdsIdentifier::SystemSupplierEcuSoftwareNumber => 0xF194,
            UdsIdentifier::SystemSupplierEcuSoftwareVersionNumber => 0xF195,
            UdsIdentifier::ExhaustRegulationOrTypeApprovalNumber => 0xF196,
            UdsIdentifier::SystemNameOrEngineType => 0xF197,
            UdsIdentifier::RepairShopOrTesterSerialNumber => 0xF198,
            UdsIdentifier::ProgrammingDate => 0xF199,
            UdsIdentifier::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber => 0xF19A,
            UdsIdentifier::CalibrationDate => 0xF19B,
            UdsIdentifier::CalibrationEquipmentSoftwareNumber => 0xF19C,
            UdsIdentifier::EcuInstallationDate => 0xF19D,
            UdsIdentifier::OdxFile => 0xF19E,
            UdsIdentifier::Entity => 0xF19F,
            UdsIdentifier::UdsVersionData => 0xFF00,
            UdsIdentifier::ReservedForIso15765_5 => 0xFF01,
        }
    }
}

impl core::fmt::Display for UdsIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let value: u16 = (*self).into();
        write!(f, "{value:#06X?}")
    }
}

impl core::fmt::Debug for UdsIdentifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UdsRoutineIdentifier {
    /// ISO/SAE reserved routine identifier (`0x0000–0x00FF`, `0xE300–0xEFFF`, `0xFF02–0xFFFF`).
    IsoSaeReserved(u16),
    /// Represent Tachograph test result values
    ///
    /// 0x0100-0x01FF
    TachographTestIds(u16),

    /// Vehicle Manufacturer Specific Routine Identifiers
    ///
    /// 0x0200-0xDFFF
    VehicleManufacturerSpecific(u16),

    /// 0xE000-0xE1FF
    ObdTestIds(u16),

    /// Execute Service Programming Loop (SPL)
    ///
    /// 0xE200
    ExecuteSpl,

    /// Deploy Loop Routine ID
    ///
    /// 0xE201
    DeployLoopRoutineId,

    /// 0xE202-0xE2FF
    SafetySystemRoutineId(u16),

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

/// We know all values for the Routine Identifier, so we can implement `From<u16>` for `UdsRoutineIdentifier`
impl From<u16> for UdsRoutineIdentifier {
    fn from(value: u16) -> Self {
        match value {
            0x0000..=0x00FF | 0xE300..=0xEFFF | 0xFF02..=0xFFFF => Self::IsoSaeReserved(value),
            0x0100..=0x01FF => Self::TachographTestIds(value),
            0x0200..=0xDFFF => Self::VehicleManufacturerSpecific(value),
            0xE000..=0xE1FF => Self::ObdTestIds(value),
            0xE200 => Self::ExecuteSpl,
            0xE201 => Self::DeployLoopRoutineId,
            0xE202..=0xE2FF => Self::SafetySystemRoutineId(value),
            0xF000..=0xFEFF => Self::SystemSupplierSpecific(value),
            0xFF00 => Self::EraseMemory,
            0xFF01 => Self::CheckProgrammingDependencies,
        }
    }
}

impl From<UdsRoutineIdentifier> for u16 {
    #[allow(clippy::match_same_arms)]
    fn from(value: UdsRoutineIdentifier) -> Self {
        match value {
            UdsRoutineIdentifier::IsoSaeReserved(identifier) => identifier,
            UdsRoutineIdentifier::TachographTestIds(identifier) => identifier,
            UdsRoutineIdentifier::VehicleManufacturerSpecific(identifier) => identifier,
            UdsRoutineIdentifier::ObdTestIds(identifier) => identifier,
            UdsRoutineIdentifier::ExecuteSpl => 0xE200,
            UdsRoutineIdentifier::DeployLoopRoutineId => 0xE201,
            UdsRoutineIdentifier::SafetySystemRoutineId(identifier) => identifier,
            UdsRoutineIdentifier::SystemSupplierSpecific(identifier) => identifier,
            UdsRoutineIdentifier::EraseMemory => 0xFF00,
            UdsRoutineIdentifier::CheckProgrammingDependencies => 0xFF01,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uds_identifier_from_is_total_and_round_trips() {
        // Every u16 maps to a variant and round-trips back to itself.
        for raw in 0u16..=u16::MAX {
            let id = UdsIdentifier::from(raw);
            assert_eq!(u16::from(id), raw, "round-trip failed for {raw:#06X}");
        }
    }

    #[test]
    fn uds_routine_identifier_from_is_total_and_round_trips() {
        // Every u16 maps to a variant and round-trips back to itself.
        for raw in 0u16..=u16::MAX {
            let id = UdsRoutineIdentifier::from(raw);
            assert_eq!(u16::from(id), raw, "round-trip failed for {raw:#06X}");
        }
    }

    #[test]
    fn uds_identifier_classifies_representative_ranges() {
        use UdsIdentifier::*;
        assert!(matches!(
            UdsIdentifier::from(0x0042),
            IsoSaeReserved(0x0042)
        ));
        assert!(matches!(
            UdsIdentifier::from(0x2000),
            VehicleManufacturerSpecific(0x2000)
        ));
        assert!(matches!(
            UdsIdentifier::from(0xA600),
            ReservedForLegislativeUse(0xA600)
        ));
        assert!(matches!(
            UdsIdentifier::from(0xF100),
            IdentificationOptionVehicleManufacturerSpecific(0xF100)
        ));
        assert!(matches!(UdsIdentifier::from(0xF190), Vin));
        assert!(matches!(
            UdsIdentifier::from(0xF200),
            PeriodicDataIdentifier(0xF200)
        ));
        assert!(matches!(
            UdsIdentifier::from(0xF400),
            ObdDataIdentifier(0xF400)
        ));
        assert!(matches!(UdsIdentifier::from(0xFA10), NumberOfEdrDevices));
        assert!(matches!(UdsIdentifier::from(0xFF00), UdsVersionData));
        assert!(matches!(UdsIdentifier::from(0xFF01), ReservedForIso15765_5));
        assert!(matches!(
            UdsIdentifier::from(0xFFFE),
            IsoSaeReserved(0xFFFE)
        ));
    }
}
