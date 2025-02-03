// use bitmask_enum::bitmask;
use serde::{Deserialize, Serialize};

use crate::{SingleValueWireFormat, WireFormat};

/// DTCStatusMask (1 byte)
/// 8 DTC status bits. Refer to D.2
/// A DTC status matches the mask if any one of the DTCs actual status bits is set to `1`
/// and the corresponding on in the mask is set to 1
/// if( DTCStatusMask & DTCStatus = !0) is a match
///
/// Server note:
///     If the mask uses bits that the server does not support,
///     the server shall process the bits it does support and ignore the rest
///
/// Per DTC statuses
///
/// | DTC Status Bit | DTC Status Name | Bit state after ClearDiagnosticInformation|
/// | - | ------------------------------ | --- |
/// | 0 | [`TestFailed`](DTCStatusMask::TestFailed)                         | **0** |
/// | 1 | [`TestFailedThisOperationCycle`](DTCStatusMask::TestFailedThisOperationCycle)       | **0** |
/// | 2 | [`PendingDTC`](DTCStatusMask::PendingDTC)                         | **0** |
/// | 3 | [`ConfirmedDTC`](DTCStatusMask::ConfirmedDTC)                       | **0** |
/// | 4 | [`TestNotCompletedSinceLastClear`](DTCStatusMask::TestNotCompletedSinceLastClear)     | **1** |
/// | 5 | [`TestFailedSinceLastClear`](DTCStatusMask::TestFailedSinceLastClear)           | **0** |
/// | 6 | [`TestNotCompletedThisOperationCycle`](DTCStatusMask::TestNotCompletedThisOperationCycle) | **1** |
/// | 7 | [`WarningIndicatorRequested`](DTCStatusMask::WarningIndicatorRequested)          | **0** |
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub enum DTCStatusMask {
    /// Status of the most recently performed test.
    ///
    /// Bit state definition:
    ///     * 0 shall indicate the last test passed
    ///     * 1 shall indicate the last matured test failed
    ///
    /// Will be 0 after a successful [`ClearDiagnosticInformation`](crate::services::ClearDiagnosticInformation) service
    TestFailed,
    /// Whether or not a diagnostic test has reported a test failed result during the current operation cycle, or that it's been reported during this operation and after [`ClearDiagnosticInformation`]
    ///
    /// Bit state definition:
    ///     * 0 shall indicate that no test failed during the current operation cycle or after a ClearDiagnosticInformation
    ///    * 1 shall indicate that a test failed during the current operation cycle or after a ClearDiagnosticInformation
    ///
    /// Shall remain a 1 until a new operation cycle is started
    TestFailedThisOperationCycle,

    /// Similar to [TestFailedThisOperationCycle], but will only clear after
    /// a cycle is finished and there is a passed test w/ no failure
    ///
    /// Bit state definition:
    ///    * 0 -  Test passed with no failure after completing a cycle
    ///    * 1 -  Test failed during the current operation cycle
    PendingDTC,
    ConfirmedDTC,
    TestNotCompletedSinceLastClear,
    TestFailedSinceLastClear,
    TestNotCompletedThisOperationCycle,
    WarningIndicatorRequested,
}
impl DTCStatusMask {
    pub fn value(&self) -> u8 {
        match self {
            Self::TestFailed => 0b0000_0001,
            Self::TestFailedThisOperationCycle => 0b0000_0010,
            Self::PendingDTC => 0b0000_0100,
            Self::ConfirmedDTC => 0b0000_1000,
            Self::TestNotCompletedSinceLastClear => 0b0001_0000,
            Self::TestFailedSinceLastClear => 0b0010_0000,
            Self::TestNotCompletedThisOperationCycle => 0b0100_0000,
            Self::WarningIndicatorRequested => 0b1000_0000,
        }
    }
}

impl From<DTCStatusMask> for u8 {
    fn from(value: DTCStatusMask) -> Self {
        value.value()
    }
}

impl From<u8> for DTCStatusMask {
    fn from(value: u8) -> Self {
        match value {
            0b0000_0000 => Self::TestFailed,
            0b0000_0001 => Self::TestFailedThisOperationCycle,
            0b0000_0010 => Self::PendingDTC,
            0b0000_0011 => Self::ConfirmedDTC,
            0b0000_0100 => Self::TestNotCompletedSinceLastClear,
            0b0000_0101 => Self::TestFailedSinceLastClear,
            0b0000_0110 => Self::TestNotCompletedThisOperationCycle,
            0b0000_0111 => Self::WarningIndicatorRequested,
            _ => panic!("Invalid DTCStatus value: {value}"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum DTCFormatIdentifier {
    /// SAE J2012 DA DTC Format
    SAE_J2012_DA_DTCFormat_00 = 0x00,
    /// reported for DTCAndStatusRecord
    ISO_14229_1_DTCFormat = 0x01,
    /// Defined in SAE J1939-73
    SAE_J1939_73_DTCFormat = 0x02,

    ISO_11992_4_DTCFormat = 0x03,

    SAE_J2012_DA_DTCFormat_04 = 0x04,

    /// Reserved for future usage
    /// 0x05 - 0xFF
    ISOSAEReserved,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub struct DTCMaskRecord {
    high_byte: u8,
    middle_byte: u8,
    low_byte: u8,
}
impl DTCMaskRecord {
    pub fn new(high_byte: u8, middle_byte: u8, low_byte: u8) -> Self {
        Self {
            high_byte,
            middle_byte,
            low_byte,
        }
    }
}
impl WireFormat for DTCMaskRecord {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, crate::Error> {
        let mut buffer = [0; 3];
        reader.read_exact(&mut buffer)?;
        Ok(Some(Self {
            high_byte: buffer[0],
            middle_byte: buffer[1],
            low_byte: buffer[2],
        }))
    }

    fn required_size(&self) -> usize {
        3
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, crate::Error> {
        writer.write_all(&[self.high_byte, self.middle_byte, self.low_byte])?;
        Ok(3)
    }
}

impl SingleValueWireFormat for DTCMaskRecord {}

/// Used to distinguish commands sent by the test equipment between different functional system groups
/// within an electrical architecture which consists of many different servers.
///
/// For the purpose of:
///     * Requesting DTC status from a vehicle
///     * Clearing DTC information in the vehicle
#[derive(Debug, Clone, Eq, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum FunctionalGroupIdentifier {
    /// 0x00 to 0x32
    /// 0x34 to 0xCF
    /// 0xE0 to 0xFD
    /// 0xFF
    ISOSAEReserved,
    /// 0x33
    EmissionsSystemGroup,
    /// 0xD0
    SafetySystemGroup,

    /// 0xD1 to 0xDF
    /// For future use
    LegislativeSystemGroup,

    /// 0xFE
    VODBSystem,
}

impl FunctionalGroupIdentifier {
    pub fn value(&self) -> u8 {
        match self {
            FunctionalGroupIdentifier::EmissionsSystemGroup => 0x33,
            FunctionalGroupIdentifier::SafetySystemGroup => 0xD0,
            FunctionalGroupIdentifier::VODBSystem => 0xFE,
            FunctionalGroupIdentifier::LegislativeSystemGroup => {
                todo!("FunctionalGroupIdentifiers::LegislativeSystemGroup is not a valid value")
            }
            _ => todo!("FunctionalGroupIdentifiers::ISOSAEReserved is not a valid value"),
        }
    }
}

impl From<u8> for FunctionalGroupIdentifier {
    fn from(value: u8) -> Self {
        match value {
            0x33 => FunctionalGroupIdentifier::EmissionsSystemGroup,
            0xD0 => FunctionalGroupIdentifier::SafetySystemGroup,
            0xFE => FunctionalGroupIdentifier::VODBSystem,
            0xD1..=0xDF => FunctionalGroupIdentifier::LegislativeSystemGroup,
            _ => FunctionalGroupIdentifier::ISOSAEReserved,
        }
    }
}

impl From<FunctionalGroupIdentifier> for u8 {
    fn from(value: FunctionalGroupIdentifier) -> Self {
        value.value()
    }
}

/// Used for non-emissions related servers
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum DTCFaultDetectionCounter {}

/// GTR DTC Class Information
///
/// Bits 7-5 of the [DTCSeverityMask]/[DTCSeverity] parameters contain severity information (optional)
/// Bits 4-0 of the [DTCSeverityMask]/[DTCSeverity] parameters contain class information (mandatory)
///
/// DTCCLASS_
// #[bitmask(u8)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DTCSeverityMask {
    // GtrDtcClassInfo
    /// Unclassified
    DTCClass_0,

    /// Matches GTR module B Class A definition
    /// Malfunction is Class A when OBD threshold limits (OTL) are assumed to be exceeded
    /// It is accepted that the emissions may not be above the OTLs when this class of malfunction occurs
    DTCClass_1,

    /// Matches GTR module B Class B1 definition
    DTCClass_2,
    /// Matches GTR module B Class B2 definition
    DTCClass_3,
    /// Matches GTR module B Class C definition
    DTCClass_4,

    // DTCSeverityInfo section
    /// Failure requests maintenance only
    ///
    /// MO
    MaintenanceOnly = 0b0010_0000, // bit 5

    /// Indicates to the failure that a check of the vehicle is required at the next halt
    ///
    /// CHKANH
    CheckAtNextHalt = 0b0100_0000, // bit 6

    /// Immediate check of the vehicle is required,
    ///
    /// CHKI
    CheckImmediately = 0b1000_0000, // bit 7
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_gtr_dtc_class_info() {
//         let dtc_class = DTCSeverityMask::DTCClass_1 | DTCSeverityMask::MaintenanceOnly;
//         assert_eq!(dtc_class.bits(), 0b0010_0010);
//     }

//     #[test]
//     fn test_dtc_severity_info() {
//         let dtc_severity = DTCSeverityMask::CheckImmediately;
//         assert_eq!(dtc_severity.bits(), 0b1000_0000);
//     }
// }
