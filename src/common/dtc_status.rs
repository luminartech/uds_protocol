use bitmask_enum::bitmask;
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{Error, SingleValueWireFormat, WireFormat};

/// Bit-packed DTC status information used by the ReadDTCInformation service
///
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
/// ```
/// use uds_protocol::{DTCStatusMask, ReadDTCInfoSubFunction};
/// // Get DTCs with TestFailed and PendingDTC statuses
/// let dtc_status = DTCStatusMask::TestFailed | DTCStatusMask::PendingDTC;
/// let dtc_subfunction = ReadDTCInfoSubFunction::ReportNumberOfDTC_ByStatusMask(dtc_status);
/// ```
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
#[bitmask(u8)]
#[derive(Serialize, Deserialize)]
pub enum DTCStatusMask {
    /// Status of the most recently performed test.
    ///
    /// Bit state definition:
    /// * 0 shall indicate the last test passed
    /// * 1 shall indicate the last matured test **failed**
    ///
    /// Will be 0 after a successful [`ClearDiagnosticInformation`](crate::services::ClearDiagnosticInformation) service
    TestFailed,
    /// Whether or not a diagnostic test has reported a test failed result during the current operation cycle,
    /// or that it's been reported during this operation and after ClearDiagnosticInformation
    ///
    /// Bit state definition:
    /// * 0 shall indicate that **no test failed** during the current operation cycle or after a ClearDiagnosticInformation
    /// * 1 shall indicate that a test failed during the current operation cycle or after a ClearDiagnosticInformation
    ///
    /// Shall remain a 1 until a new operation cycle is started
    TestFailedThisOperationCycle,

    /// Similar to [Self::TestFailedThisOperationCycle], but will only clear after
    /// a cycle is finished and there is a passed test w/ no failure
    ///
    /// Bit state definition:
    /// * 0 -  Test passed **with no failure** after completing a cycle
    /// * 1 -  Test failed during the current operation cycle
    PendingDTC,

    /// Indicates whether a malfunction was detected enough times to warrant the DTC being stored
    /// in long term memory. This doesn't mean that the DTC failure is present at the time of the request.
    /// Aging threshold for clearing itself depends on the vehicle manufacturer or OBD regulations
    ///
    /// Bit state definition:
    /// * 0 - DTC has **never been confirmed** since last ClearDiagnosticInformation, or after aging criteria have been met
    /// * 1 - DTC has been confirmed at least once
    ConfirmedDTC,

    /// Indicates whether a test has run and completed since last ClearDiagnosticInformation
    /// Will not reset to 1 by any method other than calling ClearDiagnosticInformation
    ///
    /// Bit state definition:
    /// * 0 - Test has returned passed or failed at least once since last ClearDiagnosticInformation
    /// * 1 - Test has **not** run to completion
    TestNotCompletedSinceLastClear,

    /// Indicates whether a test has failed since the last ClearDiagnosticInformation
    /// This is a latched [Self::TestFailedThisOperationCycle]
    /// Vehicle manufacturer is in charge of clearing this bit if there is an aging threshold is fulfilled
    ///
    /// Bit state definition:
    /// * 0 - Test has **not** failed since last ClearDiagnosticInformation
    /// * 1 - Test has failed at least once since last ClearDiagnosticInformation
    TestFailedSinceLastClear,

    /// Indicates whether a test has run and completed during the current operation cycle,
    ///     or whether is has run and completed after the last ClearDiagnosticInformation during the current operation cycle
    ///
    /// Bit state definition:
    /// * 0 - Test has run and completed during the current operation cycle
    /// * 1 - Test has **not** run to completion during the current operation cycle
    TestNotCompletedThisOperationCycle,

    /// Shall report the status of any warning indicators associated with a certain DTC. Warning outputs may consist
    /// of indicator lamp(s), displayed text information, etc.
    ///
    /// Bit state definition:
    /// * 0 - Server is **not** requesting a warningIndicator to be active
    /// * 1 - Server is requesting a warningIndicator to be active
    WarningIndicatorRequested,
}

impl WireFormat for DTCStatusMask {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, crate::Error> {
        let status_byte = reader.read_u8()?;
        Ok(Some(Self::from(status_byte)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, crate::Error> {
        writer.write_u8(self.bits())?;
        Ok(1)
    }
}

impl SingleValueWireFormat for DTCStatusMask {}

/// Specifies the format of the DTC reported by the server.
///
/// A given server shall only support one DTCFormatIdentifier.
#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum DTCFormatIdentifier {
    /// Defined in [SAE J2012-DA](<https://www.sae.org/standards/content/j2012da_202403/>) DTC Format
    SAE_J2012_DA_DTCFormat_00 = 0x00,

    /// reported for DTCAndStatusRecord
    ISO_14229_1_DTCFormat = 0x01,

    /// Defined in [SAE J1939-73](<https://www.sae.org/standards/content/j1939/73_202208/>)
    SAE_J1939_73_DTCFormat = 0x02,

    /// Defined in [ISO-11992](<https://www.iso.org/standard/33992.html>)
    ISO_11992_4_DTCFormat = 0x03,

    /// Defined in SAE J2012-DA](<https://www.sae.org/standards/content/j2012da_202403/>)
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
impl From<u32> for DTCMaskRecord {
    fn from(value: u32) -> Self {
        Self {
            high_byte: ((value >> 16) & 0xFF) as u8,
            middle_byte: ((value >> 8) & 0xFF) as u8,
            low_byte: (value & 0xFF) as u8,
        }
    }
}

impl WireFormat for DTCMaskRecord {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, crate::Error> {
        let high_byte = match reader.read_u8() {
            Ok(byte) => byte,
            Err(_) => return Ok(None),
        };
        let middle_byte = reader.read_u8()?;
        let low_byte = reader.read_u8()?;
        Ok(Some(Self {
            high_byte,
            middle_byte,
            low_byte,
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
    ISOSAEReserved(u8),
    /// 0x33
    EmissionsSystemGroup,
    /// 0xD0
    SafetySystemGroup,

    /// 0xD1 to 0xDF
    /// For future use
    LegislativeSystemGroup(u8),

    /// 0xFE
    VODBSystem,
}

impl FunctionalGroupIdentifier {
    pub fn value(&self) -> u8 {
        match self {
            FunctionalGroupIdentifier::EmissionsSystemGroup => 0x33,
            FunctionalGroupIdentifier::SafetySystemGroup => 0xD0,
            FunctionalGroupIdentifier::VODBSystem => 0xFE,
            FunctionalGroupIdentifier::LegislativeSystemGroup(value) => {
                todo!(
                    "FunctionalGroupIdentifiers::LegislativeSystemGroup is not a valid value {}",
                    value
                )
            }
            FunctionalGroupIdentifier::ISOSAEReserved(value) => {
                todo!(
                    "FunctionalGroupIdentifiers::ISOSAEReserved is not a valid value {}",
                    value
                )
            }
        }
    }
}

impl From<u8> for FunctionalGroupIdentifier {
    fn from(value: u8) -> Self {
        match value {
            0x33 => FunctionalGroupIdentifier::EmissionsSystemGroup,
            0xD0 => FunctionalGroupIdentifier::SafetySystemGroup,
            0xFE => FunctionalGroupIdentifier::VODBSystem,
            0xD1..=0xDF => FunctionalGroupIdentifier::LegislativeSystemGroup(value),
            _ => FunctionalGroupIdentifier::ISOSAEReserved(value),
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
/// Bits 7-5 of the DTCSeverityMask/DTCSeverity parameters contain severity information (optional)
/// Bits 4-0 of the DTCSeverityMask/DTCSeverity parameters contain class information (mandatory)
///
/// DTCCLASS_
#[allow(non_camel_case_types)]
#[bitmask(u8)]
#[derive(Serialize, Deserialize)]
pub enum DTCSeverityMask {
    // GtrDtcClassInfo
    /// Unclassified
    DTCClass_0,

    /// Matches GTR module B Class A definition
    /// Malfunction is Class A when On-Board Diagnostic (OBD) threshold limits (OTL) are assumed to be exceeded
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

impl DTCSeverityMask {
    // Validate that at least one of the DTCClass bits is set
    // Multiple Class bits may be set to get info for multiple DTC classes
    pub fn is_valid(&self) -> bool {
        self.intersects(
            Self::DTCClass_0
                | Self::DTCClass_1
                | Self::DTCClass_2
                | Self::DTCClass_3
                | Self::DTCClass_4,
        )
    }
}

/// Indicates the number of the specific DTCSnapshot data record requested
/// Setting to 0xFF will return all DTCStoredDataRecords at once
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct DTCStoredDataRecordNumber(u8);

// create a constructor for DTCStoredDataRecordNumber
impl DTCStoredDataRecordNumber {
    pub fn new(record_number: u8) -> Result<Self, Error> {
        if record_number == 0 || record_number == 0xF0 {
            return Err(Error::ReservedForLegislativeUse(
                "DTCStoredDataRecordNumber".to_string(),
                record_number,
            ));
        }
        Ok(Self(record_number))
    }
}

impl WireFormat for DTCStoredDataRecordNumber {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value = reader.read_u8()?;
        match value {
            // Reserved for Legislative purposes
            0x00 => {
                return Err(Error::ReservedForLegislativeUse(
                    "DTCStoredDataRecordNumber".to_string(),
                    value,
                ))
            }
            // Requests that the server report all DTCStoredData records at once
            0xFF => {}
            _ => {}
        }
        Ok(Some(Self(value)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.0)?;
        Ok(1)
    }
}

impl SingleValueWireFormat for DTCStoredDataRecordNumber {}

impl From<u8> for DTCStoredDataRecordNumber {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

/// For subfunctions 0x06 (ReportDTCExtDataRecord_ByDTCNumber), 0x19 (ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct DTCExtDataRecordNumber(pub u8);

impl WireFormat for DTCExtDataRecordNumber {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value = reader.read_u8()?;
        Ok(Some(Self(value)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.0)?;
        Ok(1)
    }
}

impl SingleValueWireFormat for DTCExtDataRecordNumber {}

#[cfg(test)]
mod dtc_status_tests {
    use super::*;

    #[test]
    fn status_mask() {
        let status_mask = DTCStatusMask::TestFailed | DTCStatusMask::PendingDTC;
        assert_eq!(status_mask.bits(), 0b0000_0101);

        let status_mask = DTCStatusMask::TestFailedThisOperationCycle
            | DTCStatusMask::TestNotCompletedSinceLastClear;

        assert_eq!(status_mask.bits(), 0b0001_0010);
    }

    #[test]
    fn gtr_dtc_class_info() {
        let dtc_class = DTCSeverityMask::DTCClass_1 | DTCSeverityMask::MaintenanceOnly;
        assert_eq!(dtc_class.bits(), 0b0010_0010);
        assert!(dtc_class.is_valid());
    }

    #[test]
    fn dtc_severity_info() {
        let dtc_severity = DTCSeverityMask::CheckImmediately;
        assert_eq!(dtc_severity.bits(), 0b1000_0000);
    }

    #[test]
    fn dtc_mask_record() {
        let record = DTCMaskRecord::new(0x01, 0x02, 0x03);
        let mut writer = Vec::new();
        let written_number = record.to_writer(&mut writer).unwrap();
        assert_eq!(record.required_size(), 3);
        assert_eq!(written_number, 3);
    }
}
