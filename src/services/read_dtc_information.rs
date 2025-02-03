//! ReadDTCInformation (0x19) request and response service implementation
use serde::{Deserialize, Serialize};

use crate::{DTCMaskRecord, DTCSeverityMask, DTCStatusMask, FunctionalGroupIdentifier};
use crate::{SingleValueWireFormat, WireFormat};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct ReadDtcInformationRequest {
    pub status: u8,
    pub dtc: Vec<u8>,
}

impl WireFormat for ReadDtcInformationRequest {
    fn option_from_reader<T: std::io::Read>(_reader: &mut T) -> Result<Option<Self>, crate::Error> {
        todo!()
    }

    fn required_size(&self) -> usize {
        todo!()
    }

    fn to_writer<T: std::io::Write>(&self, _writer: &mut T) -> Result<usize, crate::Error> {
        todo!()
    }
}

impl SingleValueWireFormat for ReadDtcInformationRequest {}

// turn off warning
// Initialization bits for the ECU DTC statuses
// Expected to be false prior to first power-up of ECU
// shall remain at true until ECU is reset or vehicle manufacturer specific reset is performed
// These might be ECU specific, and not given over the wire
// initializationFlag_TF = TestFailed
// initializationFlag_TFTOC = TestFailedThisOperationCycle
// initializationFlag_PDTC = PendingDTC
// initializationFlag_CDTC = ConfirmedDTC
// initializationFlag_TNCSLC = TestNotCompletedSinceLastClear
// initializationFlag_TFSLC = TestFailedSinceLastClear
// initializationFlag_TNCTOC = TestNotCompletedThisOperationCycle
// initializationFlag_WIR = WarningIndicatorRequested

// Add these type definitions
type DTCSnapshotRecordNumber = u8;
type DTCStoredDataRecordNumber = u8;
type DTCExtDataRecordNumber = u8;
type UserDefDTCSnapshotRecordNumber = u8;
type MemorySelection = u8;
type DTCReadinessGroupIdentifier = u8;
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReadDTCInformationSubFunction {
    /// Parameter: DTCStatusMask
    ///
    /// 0x01
    ReportNumberOfDTC_ByStatusMask(DTCStatusMask),
    /// Parameter: DTCStatusMask
    ///
    /// 0x02
    ReportDTC_ByStatusMask(DTCStatusMask),

    /// 0x03
    ReportDTCSnapshotIdentification,

    /// Parameter: DTCMaskRecord (3 bytes)
    /// Parameter DTCSnapshotRecordNumber(1)
    ///
    /// 0x04
    ReportDTCSnapshotRecord_ByDTCNumber(DTCMaskRecord, DTCSnapshotRecordNumber),

    /// Parameter: DTCStoredDataRecordNumber(1)
    ///
    /// 0x05
    ReportDTCStoredData_ByRecordNumber(DTCStoredDataRecordNumber),

    /// Parameter: DTCMaskRecord (3 bytes)
    /// Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x06
    ReportDTCExtDataRecord_ByDTCNumber(DTCMaskRecord, DTCExtDataRecordNumber),

    /// Parameter: DTCSeverityMaskRecord(2)
    ///     * DTCSeverityMask
    ///     * DTCStatusMask
    ///
    /// 0x07
    ReportNumberOfDTC_BySeverityMaskRecord(DTCSeverityMask, DTCStatusMask),
    /// 0x08
    ReportDTC_BySeverityMaskRecord(DTCSeverityMask, DTCStatusMask),

    /// Parameter: DTCMaskRecord (3 bytes)
    /// 0x09
    ReportSeverityInfoOfDTC(DTCMaskRecord),

    /// 0x0A
    ReportSupportedDTC,
    /// 0x0B
    ReportFirstTestFailedDTC,
    /// 0x0C
    ReportFirstConfirmedDTC,
    /// 0x0D
    ReportMostRecentTestFailedDTC,
    /// 0x0E
    ReportMostRecentConfirmedDTC,
    /// 0x14
    ReportDTCFaultDetectionCounter,
    /// 0x15
    ReportDTCWithPermanentStatus,

    /// Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x16
    ReportDTCExtDataRecord_ByRecordNumber(DTCExtDataRecordNumber),

    /// Parameter: DTCStatusMask
    ///
    /// 0x17
    ReportUserDefMemoryDTC_ByStatusMask(DTCStatusMask),

    // TODO: UserDef and MemorySelection might just need to be u8
    /// 0x18
    ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(
        DTCMaskRecord,
        UserDefDTCSnapshotRecordNumber,
        MemorySelection,
    ),

    /// Parameter: DTCMaskRecord (3 bytes)
    /// Parameter: DTCExtDataRecordNumber(1)
    /// Parameter: MemorySelection(1)
    ///
    /// 0x19
    ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(
        DTCMaskRecord,
        DTCExtDataRecordNumber,
        MemorySelection,
    ), // 0x19

    /// Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x1A
    ReportSupportedDTCExtDataRecord(DTCExtDataRecordNumber), // 0x1A

    /// Parameter: FunctionalGroupIdentifier(1)
    /// Parameter: DTCStatusMask
    /// Parameter: DTCSeverityMask
    ///
    /// 0x42
    ReportWWHOBDDTC_ByMaskRecord(FunctionalGroupIdentifier, DTCStatusMask, DTCSeverityMask),

    /// Parameter: FunctionalGroupIdentifier(1)
    ///
    /// 0x55
    ReportWWHOBDDTC_WithPermanentStatus(FunctionalGroupIdentifier),

    /// Parameter: FunctionalGroupIdentifier(1)
    /// Parameter: DTCReadinessGroupIdentifier(1)
    ///
    /// 0x56
    ReportDTCInformation_ByDTCReadinessGroupIdentifier(
        FunctionalGroupIdentifier,
        DTCReadinessGroupIdentifier,
    ),
}
