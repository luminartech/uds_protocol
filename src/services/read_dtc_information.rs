//! ReadDTCInformation (0x19) request and response service implementation
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    DTCExtDataRecordList, DTCExtDataRecordNumber, DTCRecord, DTCSeverityMask, DTCSeverityRecord,
    DTCSnapshotRecordList, DTCSnapshotRecordNumber, DTCStatusMask, DTCStoredDataRecordNumber,
    Error, FunctionalGroupIdentifier, IterableWireFormat, SingleValueWireFormat,
    UserDefDTCSnapshotRecordNumber, WireFormat,
};

/// Used for non-emissions related servers
type DTCFaultDetectionCounter = u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct ReadDTCInfoRequest {
    pub dtc_subfunction: ReadDTCInfoSubFunction,
}

impl ReadDTCInfoRequest {
    pub(crate) fn new(dtc_subfunction: ReadDTCInfoSubFunction) -> Self {
        Self { dtc_subfunction }
    }
}

impl WireFormat for ReadDTCInfoRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let dtc_subfunction = ReadDTCInfoSubFunction::from_reader(reader)?;

        Ok(Some(Self { dtc_subfunction }))
    }

    fn required_size(&self) -> usize {
        self.dtc_subfunction.required_size()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.dtc_subfunction.to_writer(writer)
    }
}

impl SingleValueWireFormat for ReadDTCInfoRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
pub struct DTCFaultDetectionCounterRecord {
    pub dtc_record: DTCRecord,
    pub dtc_fault_detection_counter: DTCFaultDetectionCounter,
}

impl WireFormat for DTCFaultDetectionCounterRecord {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let dtc_record = match DTCRecord::option_from_reader(reader) {
            Ok(None) => return Ok(None),
            Ok(record) => record,
            Err(_) => return Ok(None),
        };
        let dtc_fault_detection_counter = reader.read_u8()?;
        Ok(Some(Self {
            dtc_record: dtc_record.unwrap(),
            dtc_fault_detection_counter,
        }))
    }

    fn required_size(&self) -> usize {
        4
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.dtc_record.to_writer(writer)?;
        writer.write_u8(self.dtc_fault_detection_counter)?;
        Ok(self.required_size())
    }
}

impl IterableWireFormat for DTCFaultDetectionCounterRecord {}

/// Used to address the respective user-defined DTC memory when retrieving DTCs
type MemorySelection = u8;
/// Have to reference SAE J1979-DA for the corresponding DTC readiness groups and the [FunctionalGroupIdentifier]s
/// This RGID depends on the functional group
type DTCReadinessGroupIdentifier = u8; // RGID

/// Subfunctions for the ReadDTCInformation service
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReadDTCInfoSubFunction {
    /// * Parameter: DTCStatusMask
    ///
    /// 0x01
    ReportNumberOfDTC_ByStatusMask(DTCStatusMask),
    /// * Parameter: DTCStatusMask
    ///
    /// 0x02
    ReportDTC_ByStatusMask(DTCStatusMask),

    /// 0x03
    ReportDTCSnapshotIdentification,

    /// Parameter: DTCRecord (3 bytes)
    /// Parameter DTCSnapshotRecordNumber(1)
    ///
    /// 0x04
    ReportDTCSnapshotRecord_ByDTCNumber(DTCRecord, DTCSnapshotRecordNumber),

    /// * Parameter: DTCStoredDataRecordNumber(1)
    ///
    /// 0x05
    ReportDTCStoredData_ByRecordNumber(DTCStoredDataRecordNumber),

    /// Parameter: DTCRecord (3 bytes)
    /// Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x06
    ReportDTCExtDataRecord_ByDTCNumber(DTCRecord, DTCExtDataRecordNumber),

    /// * Parameter: DTCSeverityMaskRecord(2)
    ///     * DTCSeverityMask
    ///     * DTCStatusMask
    ///
    /// 0x07
    ReportNumberOfDTC_BySeverityMaskRecord(DTCSeverityMask, DTCStatusMask),
    /// 0x08
    ReportDTC_BySeverityMaskRecord(DTCSeverityMask, DTCStatusMask),

    /// Parameter: DTCRecord (3 bytes)
    ///
    /// 0x09
    ReportSeverityInfoOfDTC(DTCRecord),

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

    /// * Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x16
    ReportDTCExtDataRecord_ByRecordNumber(DTCExtDataRecordNumber),

    /// * Parameter: DTCStatusMask
    ///
    /// 0x17
    ReportUserDefMemoryDTC_ByStatusMask(DTCStatusMask),

    // TODO: UserDef and MemorySelection might just need to be u8
    /// 0x18
    ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(
        DTCRecord,
        UserDefDTCSnapshotRecordNumber,
        MemorySelection,
    ),

    /// Parameter: DTCRecord (3 bytes)
    /// Parameter: DTCExtDataRecordNumber(1) (0xFF for all records)
    /// Parameter: MemorySelection(1)
    ///
    /// 0x19
    ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(
        DTCRecord,
        DTCExtDataRecordNumber,
        MemorySelection,
    ),

    /// * Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x1A
    ReportSupportedDTCExtDataRecord(DTCExtDataRecordNumber),

    /// * Parameter: FunctionalGroupIdentifier(1)
    /// * Parameter: DTCStatusMask
    /// * Parameter: DTCSeverityMask
    ///
    /// 0x42
    ReportWWHOBDDTC_ByMaskRecord(FunctionalGroupIdentifier, DTCStatusMask, DTCSeverityMask),

    /// * Parameter: FunctionalGroupIdentifier(1)
    ///
    /// 0x55
    ReportWWHOBDDTC_WithPermanentStatus(FunctionalGroupIdentifier),

    /// * Parameter: FunctionalGroupIdentifier(1)
    /// * Parameter: DTCReadinessGroupIdentifier(1)
    ///
    /// 0x56
    ReportDTCInformation_ByDTCReadinessGroupIdentifier(
        FunctionalGroupIdentifier,
        DTCReadinessGroupIdentifier,
    ),
    /// 0x42-0x54, 0x57-0x7F
    ISOSAEReserved(u8),
}

impl ReadDTCInfoSubFunction {
    pub fn value(&self) -> u8 {
        match self {
            Self::ReportNumberOfDTC_ByStatusMask(_) => 0x01,
            Self::ReportDTC_ByStatusMask(_) => 0x02,
            Self::ReportDTCSnapshotIdentification => 0x03,
            Self::ReportDTCSnapshotRecord_ByDTCNumber(_, _) => 0x04,
            Self::ReportDTCStoredData_ByRecordNumber(_) => 0x05,
            Self::ReportDTCExtDataRecord_ByDTCNumber(_, _) => 0x06,
            Self::ReportNumberOfDTC_BySeverityMaskRecord(_, _) => 0x07,
            Self::ReportDTC_BySeverityMaskRecord(_, _) => 0x08,
            Self::ReportSeverityInfoOfDTC(_) => 0x09,
            Self::ReportSupportedDTC => 0x0A,
            Self::ReportFirstTestFailedDTC => 0x0B,
            Self::ReportFirstConfirmedDTC => 0x0C,
            Self::ReportMostRecentTestFailedDTC => 0x0D,
            Self::ReportMostRecentConfirmedDTC => 0x0E,
            Self::ReportDTCFaultDetectionCounter => 0x14,
            Self::ReportDTCWithPermanentStatus => 0x15,
            Self::ReportDTCExtDataRecord_ByRecordNumber(_) => 0x16,
            Self::ReportUserDefMemoryDTC_ByStatusMask(_) => 0x17,
            Self::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(_, _, _) => 0x18,
            Self::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(_, _, _) => 0x19,
            Self::ReportSupportedDTCExtDataRecord(_) => 0x1A,
            Self::ReportWWHOBDDTC_ByMaskRecord(_, _, _) => 0x42,
            Self::ReportWWHOBDDTC_WithPermanentStatus(_) => 0x55,
            Self::ReportDTCInformation_ByDTCReadinessGroupIdentifier(_, _) => 0x56,
            Self::ISOSAEReserved(value) => *value,
        }
    }
}

impl WireFormat for ReadDTCInfoSubFunction {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let report_type = reader.read_u8()?;

        let subfunction = match report_type {
            0x01 | 0x02 => {
                let status = DTCStatusMask::from(reader.read_u8()?);
                match report_type {
                    0x01 => Self::ReportNumberOfDTC_ByStatusMask(status),
                    0x02 => Self::ReportDTC_ByStatusMask(status),
                    _ => unreachable!(),
                }
            }
            0x03 => Self::ReportDTCSnapshotIdentification,
            0x04 => Self::ReportDTCSnapshotRecord_ByDTCNumber(
                DTCRecord::from_reader(reader)?,
                DTCSnapshotRecordNumber::from_reader(reader)?,
            ),
            0x05 => Self::ReportDTCStoredData_ByRecordNumber(
                DTCStoredDataRecordNumber::from_reader(reader)?,
            ),
            // 0xFF for all records, 0xFE for all OBD records
            0x06 => Self::ReportDTCExtDataRecord_ByDTCNumber(
                DTCRecord::from_reader(reader)?,
                DTCExtDataRecordNumber::from_reader(reader)?,
            ),
            0x07 => Self::ReportNumberOfDTC_BySeverityMaskRecord(
                DTCSeverityMask::from(reader.read_u8()?),
                DTCStatusMask::from(reader.read_u8()?),
            ),
            0x08 => Self::ReportDTC_BySeverityMaskRecord(
                DTCSeverityMask::from(reader.read_u8()?),
                DTCStatusMask::from(reader.read_u8()?),
            ),
            0x09 => Self::ReportSeverityInfoOfDTC(DTCRecord::from_reader(reader)?),
            0x0A => Self::ReportSupportedDTC,
            0x0B => Self::ReportFirstTestFailedDTC,
            0x0C => Self::ReportFirstConfirmedDTC,
            0x0D => Self::ReportMostRecentTestFailedDTC,
            0x0E => Self::ReportMostRecentConfirmedDTC,
            0x14 => Self::ReportDTCFaultDetectionCounter,
            0x15 => Self::ReportDTCWithPermanentStatus,
            0x16 => Self::ReportDTCExtDataRecord_ByRecordNumber(
                DTCExtDataRecordNumber::from_reader(reader)?,
            ),
            0x17 => {
                Self::ReportUserDefMemoryDTC_ByStatusMask(DTCStatusMask::from(reader.read_u8()?))
            }
            // 0xFF for all records
            0x18 => Self::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(
                DTCRecord::from_reader(reader)?,
                UserDefDTCSnapshotRecordNumber::from_reader(reader)?,
                reader.read_u8()?,
            ),
            0x19 => Self::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(
                DTCRecord::from_reader(reader)?,
                DTCExtDataRecordNumber::from_reader(reader)?,
                reader.read_u8()?,
            ),
            0x1A => {
                Self::ReportSupportedDTCExtDataRecord(DTCExtDataRecordNumber::from_reader(reader)?)
            }
            0x42 => Self::ReportWWHOBDDTC_ByMaskRecord(
                FunctionalGroupIdentifier::EmissionsSystemGroup,
                DTCStatusMask::TestFailed,
                DTCSeverityMask::DTCClass_4,
            ),
            0x43..=0x54 => Self::ISOSAEReserved(report_type),
            0x55 => Self::ReportWWHOBDDTC_WithPermanentStatus(FunctionalGroupIdentifier::from(
                reader.read_u8()?,
            )),
            0x56 => Self::ReportDTCInformation_ByDTCReadinessGroupIdentifier(
                FunctionalGroupIdentifier::from(reader.read_u8()?),
                reader.read_u8()?,
            ),
            0x57..=0x7F => Self::ISOSAEReserved(report_type),
            _ => return Err(Error::InvalidDtcSubfunctionType(report_type)),
        };
        Ok(Some(subfunction))
    }

    /// Each subfunction has a different size
    /// The first byte is always the subfunction report type
    fn required_size(&self) -> usize {
        1 + match self {
            Self::ReportNumberOfDTC_ByStatusMask(_) => 1,
            Self::ReportDTC_ByStatusMask(_) => 1,
            Self::ReportDTCSnapshotIdentification => 0,
            Self::ReportDTCSnapshotRecord_ByDTCNumber(_, _) => 4,
            Self::ReportDTCStoredData_ByRecordNumber(_) => 2,
            Self::ReportDTCExtDataRecord_ByDTCNumber(_, _) => 4,
            Self::ReportNumberOfDTC_BySeverityMaskRecord(_, _) => 2,
            Self::ReportDTC_BySeverityMaskRecord(_, _) => 2,
            Self::ReportSeverityInfoOfDTC(_) => 3,
            Self::ReportSupportedDTC => 0,
            Self::ReportFirstTestFailedDTC => 0,
            Self::ReportFirstConfirmedDTC => 0,
            Self::ReportMostRecentTestFailedDTC => 0,
            Self::ReportMostRecentConfirmedDTC => 0,
            Self::ReportDTCFaultDetectionCounter => 0,
            Self::ReportDTCWithPermanentStatus => 0,
            Self::ReportDTCExtDataRecord_ByRecordNumber(_) => 1,
            Self::ReportUserDefMemoryDTC_ByStatusMask(_) => 1,
            Self::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(_, _, _) => 5,
            Self::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(_, _, _) => 5,
            Self::ReportSupportedDTCExtDataRecord(_) => 1,
            Self::ReportWWHOBDDTC_ByMaskRecord(_, _, _) => 3,
            Self::ReportWWHOBDDTC_WithPermanentStatus(_) => 1,
            Self::ReportDTCInformation_ByDTCReadinessGroupIdentifier(_, _) => 2,

            Self::ISOSAEReserved(_) => 0,
        }
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Write the subfunction value
        writer.write_u8(self.value())?;
        match self {
            Self::ReportNumberOfDTC_ByStatusMask(mask) => {
                mask.to_writer(writer)?;
            }
            Self::ReportDTC_ByStatusMask(mask) => {
                mask.to_writer(writer)?;
            }
            Self::ReportDTCSnapshotIdentification => {}
            Self::ReportDTCSnapshotRecord_ByDTCNumber(mask, record_number) => {
                mask.to_writer(writer)?;
                record_number.to_writer(writer)?;
            }
            Self::ReportDTCStoredData_ByRecordNumber(record_number) => {
                record_number.to_writer(writer)?;
            }
            Self::ReportDTCExtDataRecord_ByDTCNumber(mask, record_number) => {
                mask.to_writer(writer)?;
                record_number.to_writer(writer)?;
            }
            Self::ReportNumberOfDTC_BySeverityMaskRecord(severity, status) => {
                writer.write_u8(severity.bits())?;
                status.to_writer(writer)?;
            }
            Self::ReportDTC_BySeverityMaskRecord(severity, status) => {
                writer.write_u8(severity.bits())?;
                status.to_writer(writer)?;
            }
            Self::ReportSeverityInfoOfDTC(mask) => {
                mask.to_writer(writer)?;
            }
            Self::ReportSupportedDTC => {}
            Self::ReportFirstTestFailedDTC => {}
            Self::ReportFirstConfirmedDTC => {}
            Self::ReportMostRecentTestFailedDTC => {}
            Self::ReportMostRecentConfirmedDTC => {}
            Self::ReportDTCFaultDetectionCounter => {}
            Self::ReportDTCWithPermanentStatus => {}
            Self::ReportDTCExtDataRecord_ByRecordNumber(record_number) => {
                record_number.to_writer(writer)?;
            }
            Self::ReportUserDefMemoryDTC_ByStatusMask(mask) => {
                mask.to_writer(writer)?;
            }
            Self::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(mask, number, selection) => {
                mask.to_writer(writer)?;
                number.to_writer(writer)?;
                writer.write_u8(*selection)?;
            }
            Self::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(mask, number, selection) => {
                mask.to_writer(writer)?;
                number.to_writer(writer)?;
                writer.write_u8(*selection)?;
            }
            Self::ReportSupportedDTCExtDataRecord(number) => {
                number.to_writer(writer)?;
            }
            Self::ReportWWHOBDDTC_ByMaskRecord(group, status, severity) => {
                writer.write_u8(group.value())?;
                status.to_writer(writer)?;
                writer.write_u8(severity.bits())?;
            }
            Self::ReportWWHOBDDTC_WithPermanentStatus(group) => {
                writer.write_u8(group.value())?;
            }
            Self::ReportDTCInformation_ByDTCReadinessGroupIdentifier(group, readiness) => {
                writer.write_u8(group.value())?;
                writer.write_u8(*readiness)?;
            }
            Self::ISOSAEReserved(value) => {
                writer.write_u8(*value)?;
            }
        }
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for ReadDTCInfoSubFunction {}

type NumberOfDTCs = u16;
/// Same representation as [DTCStatusMask] but with the bits 'on' representing the DTC status supported by the server
/// IE if the server doesn't support [DTCStatusMask::WarningIndicatorRequested] then the bit for that status will be 'off'
/// and all other bits will be 'on'
type DTCStatusAvailabilityMask = DTCStatusMask;

/// Subfunction ID for the response
type SubFunctionID = u8;

/// Response payloads can be shared among multiple request subfunctions
///
/// For example, subfunction 0x01 and 0x07 both return the number of DTCs
/// and have the same response format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum ReadDTCInfoResponse<UserPayload> {
    /// * Parameter: [`DTCStatusAvailabilityMask`] (1)
    /// * Parameter: `NumberOfDTCs`(2)
    ///
    /// For subfunctions 0x01, 0x07
    ///   * 0x01: [ReadDTCInfoSubFunction::ReportNumberOfDTC_ByStatusMask]
    ///   * 0x07: [ReadDTCInfoSubFunction::ReportNumberOfDTC_BySeverityMaskRecord]
    NumberOfDTCs(SubFunctionID, DTCStatusAvailabilityMask, NumberOfDTCs),

    /// A list of DTCs matching the subfunction request
    ///
    /// * Parameter: [`DTCStatusAvailabilityMask`] (1)
    /// * Parameter: `Vec<DTCAndStatusRecord>` (4 * n)
    ///
    /// Note: DTC list can be empty if there are none to report,
    ///       but the response will still be sent
    ///
    /// For subfunctions 0x02, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x15
    ///   * 0x02: [ReadDTCInfoSubFunction::ReportDTC_ByStatusMask]
    ///   * 0x0A: [ReadDTCInfoSubFunction::ReportSupportedDTC]
    ///   * 0x0B: [ReadDTCInfoSubFunction::ReportFirstTestFailedDTC]
    ///   * 0x0C: [ReadDTCInfoSubFunction::ReportFirstConfirmedDTC]
    ///   * 0x0D: [ReadDTCInfoSubFunction::ReportMostRecentTestFailedDTC]
    ///   * 0x0E: [ReadDTCInfoSubFunction::ReportMostRecentConfirmedDTC]
    ///   * 0x15: [ReadDTCInfoSubFunction::ReportDTCWithPermanentStatus]
    DTCList(
        SubFunctionID,
        DTCStatusAvailabilityMask,
        Vec<(DTCRecord, DTCStatusMask)>,
    ),

    /// Snapshot identification - aka "Freeze Frame"
    ///
    /// Parameter: Vec<(DTCRecord, DTCSnapshotRecordNumber> (4 * n)
    ///
    /// Note: DTCSnapshot list might be empty
    ///
    /// For subfunction 0x03
    ///     * 0x03: [ReadDTCInfoSubFunction::ReportDTCSnapshotIdentification]
    DTCSnapshotList(Vec<(DTCRecord, DTCSnapshotRecordNumber)>),

    /// Get the DTC status and snapshot number and information w/ corresponding Data Identifier (DID)
    ///
    /// DTC, Status, snapshot number, # of identifiers, DID (times # of identifiers), Snapshot info.
    ///
    /// If all records are requested, it can be a theoretically large amount of data.
    ///
    /// Parameter: DTCRecord (3 bytes) - Echo of the request
    /// Parameter: DTCStatusMask (1) - status of the requested DTC
    /// C2/C4: There are multiple dataIdentifier/snapshotData combinations allowed to be present in a single DTCSnapshotRecord.
    /// This can, for example be the case for the situation where a single dataIdentifier only references an integral part of data. When
    /// the dataIdentifier references a block of data then a single dataIdentifier/snapshotData combination can be used.
    ///
    /// Note: See example 12.3.5.6.2 in ISO 14229-1:2020 for more information
    ///
    /// For subfunction 0x04
    ///   * 0x04: [ReadDTCInfoSubFunction::ReportDTCSnapshotRecord_ByDTCNumber]
    DTCSnapshotRecordList(DTCSnapshotRecordList<UserPayload>),

    /// List of [crate::DTCExtDataRecord]s for a given DTC.
    ///
    /// UserPayload is so the data can be read according to a specific format
    /// defined by the supplier/vehicle manufacturer
    ///
    /// * Parameter: [`DTCMaskRecord`] (3 bytes) - Echo of the request
    /// * Parameter: [`DTCStatusMask`] (1) - status of the requested DTC
    /// * Parameter: [`crate::DTCExtDataRecord`] (n)
    ///
    /// For subfunction 0x06
    ///   * 0x06: [ReadDTCInfoSubFunction::ReportDTCExtDataRecord_ByDTCNumber]
    DTCExtDataRecordList(DTCExtDataRecordList<UserPayload>),

    /// List of DTC Records that either match a severity and status mask for subfunction [ReadDTCInfoSubFunction::ReportDTC_BySeverityMaskRecord],
    /// or a single record if the request type was [ReadDTCInfoSubFunction::ReportSeverityInfoOfDTC].
    ///
    /// * Parameter: [`DTCStatusAvailabilityMask`] (1 byte)
    /// * Parameter: [`Vec<DTCSeverityRecord>`] (6 bytes)
    ///
    /// For Subfunctions 0x08, 0x09
    ///     * 0x08: [ReadDTCInfoSubFunction::ReportDTC_BySeverityMaskRecord]
    ///     * 0x09: [ReadDTCInfoSubFunction::ReportSeverityInfoOfDTC]
    DTCSeverityRecordList(
        SubFunctionID,
        DTCStatusAvailabilityMask,
        Vec<DTCSeverityRecord>,
    ),
    /// List of DTC Records along with their fault detection counters for subfunction [ReadDTCInfoSubFunction::ReportDTCFaultDetectionCounter].

    ///
    /// * Parameter: [`DTCRecord`] - (3 bytes)
    /// * Parameter: [`DTCFaultDetectionCounter`] - (1 byte)
    ///
    /// For Subfunction 0x14:
    ///     * 0x14: [ReadDTCInfoSubFunction::ReportDTCFaultDetectionCounter]
    DTCFaultDetectionCounterRecordList(Vec<DTCFaultDetectionCounterRecord>),
}

impl<UserPayload: IterableWireFormat> WireFormat for ReadDTCInfoResponse<UserPayload> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let subfunction_id = reader.read_u8()?;

        match subfunction_id {
            0x01 | 0x07 => {
                let status = DTCStatusAvailabilityMask::from(reader.read_u8()?);
                let count = reader.read_u16::<byteorder::BigEndian>()?;
                Ok(Some(Self::NumberOfDTCs(subfunction_id, status, count)))
            }
            0x02 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x15 => {
                let status = DTCStatusAvailabilityMask::from(reader.read_u8()?);
                let mut dtcs: Vec<(DTCRecord, DTCStatusMask)> = Vec::new();

                // Loop until we're done with the reader and fill the DTC list
                while let Some(record) = DTCRecord::option_from_reader(reader)? {
                    match reader.read_u8() {
                        Ok(status) => dtcs.push((record, DTCStatusMask::from(status))),
                        Err(_) => break,
                    }
                }

                Ok(Some(Self::DTCList(subfunction_id, status, dtcs)))
            }
            0x03 => {
                let mut dtcs: Vec<(DTCRecord, DTCSnapshotRecordNumber)> = Vec::new();

                // Loop until we're done with the reader and fill the DTC list
                while let Some(record) = DTCRecord::option_from_reader(reader)? {
                    match DTCSnapshotRecordNumber::option_from_reader(reader)? {
                        Some(number) => dtcs.push((record, number)),
                        None => break,
                    }
                }

                Ok(Some(Self::DTCSnapshotList(dtcs)))
            }
            0x04 => {
                let snapshot_list = DTCSnapshotRecordList::option_from_reader(reader)?.unwrap();
                Ok(Some(Self::DTCSnapshotRecordList(snapshot_list)))
            }
            0x06 => {
                let ext_data_list = DTCExtDataRecordList::option_from_reader(reader)?.unwrap();
                Ok(Some(Self::DTCExtDataRecordList(ext_data_list)))
            }
            0x08 | 0x09 => {
                let status = DTCStatusAvailabilityMask::from(reader.read_u8()?);
                let mut dtcs = Vec::new();

                for dtc_severity_record in DTCSeverityRecord::from_reader_iterable(reader) {
                    match dtc_severity_record {
                        Ok(p) => {
                            dtcs.push(p);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(Some(Self::DTCSeverityRecordList(
                    subfunction_id,
                    status,
                    dtcs,
                )))
            }
            0x14 => {
                let mut dtcs = Vec::new();
                for dtc_fault_record in DTCFaultDetectionCounterRecord::from_reader_iterable(reader)
                {
                    match dtc_fault_record {
                        Ok(p) => {
                            dtcs.push(p);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                Ok(Some(Self::DTCFaultDetectionCounterRecordList(dtcs)))
            }

            _ => todo!(), // _ => Err(Error::InvalidDtcSubfunctionType(subfunction_id)),
        }
    }

    fn required_size(&self) -> usize {
        // subfunction ID + subfunction contents
        1 + match self {
            Self::NumberOfDTCs(_, _, _) => 3,
            Self::DTCList(_, _, list) => 1 + list.len() * 4,
            Self::DTCSnapshotList(list) => 1 + list.len() * 4,
            Self::DTCSnapshotRecordList(list) => list.required_size(),
            Self::DTCExtDataRecordList(list) => list.required_size(),
            Self::DTCSeverityRecordList(_, _, list) => 1 + list.len() * 6,
            Self::DTCFaultDetectionCounterRecordList(list) => list.len() * 4,
        }
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        match self {
            Self::NumberOfDTCs(id, mask, count) => {
                writer.write_u8(*id)?;
                writer.write_u8(mask.bits())?;
                writer.write_u16::<byteorder::BigEndian>(*count)?;
            }
            Self::DTCList(id, mask, list) => {
                writer.write_u8(*id)?;
                writer.write_u8(mask.bits())?;
                for (record, status) in list {
                    record.to_writer(writer)?;
                    status.to_writer(writer)?;
                }
            }
            Self::DTCSnapshotList(list) => {
                writer.write_u8(0x03)?;
                for (record, number) in list {
                    record.to_writer(writer)?;
                    number.to_writer(writer)?;
                }
            }
            Self::DTCSnapshotRecordList(list) => {
                writer.write_u8(0x04)?;
                list.to_writer(writer)?;
            }
            Self::DTCExtDataRecordList(list) => {
                writer.write_u8(0x06)?;
                list.to_writer(writer)?;
            }
            Self::DTCFaultDetectionCounterRecordList(list) => {
                writer.write_u8(0x14)?;
                for fault_detection_counter in list {
                    fault_detection_counter.to_writer(writer)?;
                }
            }
            Self::DTCSeverityRecordList(id, status, list) => {
                writer.write_u8(*id)?;
                status.to_writer(writer)?;
                for dtcs in list {
                    dtcs.to_writer(writer)?;
                }
            }
        }
        Ok(self.required_size())
    }
}

impl<UserPayload: IterableWireFormat> SingleValueWireFormat for ReadDTCInfoResponse<UserPayload> {}

#[cfg(test)]
mod response {

    use super::*;

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
    pub enum TestIdentifier {
        Abracadabra = 0xBEEF,
    }

    impl PartialEq<u16> for TestIdentifier {
        fn eq(&self, other: &u16) -> bool {
            match self {
                TestIdentifier::Abracadabra => *other == 0xBEEF,
            }
        }
    }

    impl WireFormat for TestIdentifier {
        fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;

            let id = u16::from_be_bytes(buf);
            if TestIdentifier::Abracadabra == id {
                Ok(Some(TestIdentifier::Abracadabra))
            } else {
                Err(Error::NoDataAvailable)
            }
        }

        fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
            writer.write_u16::<byteorder::BigEndian>(*self as u16)?;
            Ok(self.required_size())
        }

        fn required_size(&self) -> usize {
            2
        }
    }

    impl IterableWireFormat for TestIdentifier {}

    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
    enum TestPayload {
        Abracadabra(u8),
    }

    impl WireFormat for TestPayload {
        fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;

            let value = u16::from_be_bytes(buf);

            if value == TestIdentifier::Abracadabra as u16 {
                let mut byte = [0u8; 1];
                reader.read_exact(&mut byte)?;
                Ok(Some(TestPayload::Abracadabra(byte[0])))
            } else {
                Err(Error::NoDataAvailable)
            }
        }

        fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
            let id_bytes: u16 = match self {
                TestPayload::Abracadabra(_) => 0xBEEF,
            };

            writer.write_all(&id_bytes.to_be_bytes())?;

            match self {
                TestPayload::Abracadabra(value) => {
                    writer.write_u8(*value)?;
                    Ok(self.required_size())
                }
            }
        }

        fn required_size(&self) -> usize {
            3
        }
    }
    impl IterableWireFormat for TestPayload {}

    #[test]
    fn dtc_list() {
        // skip formatting
        #[rustfmt::skip]
        let bytes = [
            0x02, // subfunction
            0x01, // Availability mask
            // First DTC record
            0x01, 0x02, 0x03, (DTCStatusMask::PendingDTC | DTCStatusMask::TestFailed).into(),
            // Second DTC record
            0x17, 0x04, 0x03, DTCStatusMask::TestNotCompletedThisOperationCycle.into(),
        ];
        let mut reader = &bytes[..];
        let response: ReadDTCInfoResponse<TestPayload> =
            ReadDTCInfoResponse::from_reader(&mut reader).unwrap();
        assert_eq!(
            response,
            ReadDTCInfoResponse::DTCList(
                0x02,
                DTCStatusMask::TestFailed,
                vec![
                    (
                        DTCRecord::new(0x01, 0x02, 0x03),
                        DTCStatusMask::PendingDTC | DTCStatusMask::TestFailed
                    ),
                    (
                        DTCRecord::new(0x17, 0x04, 0x03),
                        DTCStatusMask::TestNotCompletedThisOperationCycle
                    )
                ]
            )
        );

        // write
        let mut writer = Vec::new();
        let written = response.to_writer(&mut writer).unwrap();
        assert_eq!(writer, bytes);
        assert_eq!(written, bytes.len());
        assert_eq!(written, response.required_size());
    }

    #[test]
    fn severity_list_test() {
        let bytes: [u8; 8] = [
            0x08, // subfunction
            0x01, // Availability mask
            DTCSeverityMask::CheckImmediately.into(),
            FunctionalGroupIdentifier::EmissionsSystemGroup.into(),
            0x01,
            0x02,
            0x03,
            (DTCStatusMask::PendingDTC | DTCStatusMask::TestFailed).into(),
        ];
        let mut reader = &bytes[..];
        let response: ReadDTCInfoResponse<TestPayload> =
            ReadDTCInfoResponse::from_reader(&mut reader).unwrap();
        assert_eq!(
            response,
            ReadDTCInfoResponse::DTCSeverityRecordList(
                0x08,
                DTCStatusMask::TestFailed,
                vec![
                    (DTCSeverityRecord {
                        severity: DTCSeverityMask::CheckImmediately,
                        functional_group_identifier:
                            FunctionalGroupIdentifier::EmissionsSystemGroup,
                        dtc_record: DTCRecord::new(0x01, 0x02, 0x03),
                        dtc_status_mask: (DTCStatusMask::PendingDTC | DTCStatusMask::TestFailed),
                    })
                ]
            )
        );

        // write
        let mut writer = Vec::new();
        let written = response.to_writer(&mut writer).unwrap();
        assert_eq!(writer, bytes);
        assert_eq!(written, bytes.len());
        assert_eq!(written, response.required_size());
    }

    #[test]
    fn severity_empty_list_test() {
        let bytes: [u8; 2] = [
            0x08, // subfunction
            0x01, // Availability mask
        ];
        let mut reader = &bytes[..];
        let response: ReadDTCInfoResponse<TestPayload> =
            ReadDTCInfoResponse::from_reader(&mut reader).unwrap();
        assert_eq!(
            response,
            ReadDTCInfoResponse::DTCSeverityRecordList(0x08, DTCStatusMask::TestFailed, vec![])
        );

        // write
        let mut writer = Vec::new();
        let written = response.to_writer(&mut writer).unwrap();
        assert_eq!(writer, bytes);
        assert_eq!(written, bytes.len());
        assert_eq!(written, response.required_size());
    }

    #[test]
    fn fault_detection_test() {
        let bytes = [
            0x14, // subfunction
            0x01, 0x02, 0x03, //DTC Record
            0x04, //DTC Status
        ];
        let mut reader = &bytes[..];
        let response: ReadDTCInfoResponse<TestPayload> =
            ReadDTCInfoResponse::from_reader(&mut reader).unwrap();
        assert_eq!(
            response,
            ReadDTCInfoResponse::DTCFaultDetectionCounterRecordList(vec![
                DTCFaultDetectionCounterRecord {
                    dtc_record: DTCRecord::new(0x01, 0x02, 0x03),
                    dtc_fault_detection_counter: 0x04
                }
            ])
        );

        // write
        let mut writer = Vec::new();
        let written = response.to_writer(&mut writer).unwrap();
        assert_eq!(writer, bytes);
        assert_eq!(written, bytes.len());
        assert_eq!(written, response.required_size());
    }
    #[test]
    fn fault_detection_empty_test() {
        let bytes = [
            0x14, // subfunction
        ];
        let mut reader = &bytes[..];
        let response: ReadDTCInfoResponse<TestPayload> =
            ReadDTCInfoResponse::from_reader(&mut reader).unwrap();
        assert_eq!(
            response,
            ReadDTCInfoResponse::DTCFaultDetectionCounterRecordList(vec![])
        );

        // write
        let mut writer = Vec::new();
        let written = response.to_writer(&mut writer).unwrap();
        assert_eq!(writer, bytes);
        assert_eq!(written, bytes.len());
        assert_eq!(written, response.required_size());
    }
}

#[cfg(test)]
mod ext_data {
    use super::*;

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
    pub enum TestDTCExtDataRecordNumber {
        // DTC records
        WarmUpCycleCount = 0x04,
        FaultDetectionCounter = 0x05,
    }

    impl WireFormat for TestDTCExtDataRecordNumber {
        fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
            let id = reader.read_u8();
            match id {
                Ok(0x04) => Ok(Some(TestDTCExtDataRecordNumber::WarmUpCycleCount)),
                Ok(0x05) => Ok(Some(TestDTCExtDataRecordNumber::FaultDetectionCounter)),
                Err(_) => Ok(None),
                _ => Err(Error::NoDataAvailable),
            }
        }

        fn required_size(&self) -> usize {
            1
        }

        fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
            writer.write_u8(*self as u8)?;
            Ok(self.required_size())
        }
    }

    impl IterableWireFormat for TestDTCExtDataRecordNumber {}

    #[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
    enum TestDTCExtData {
        WarmUpCycleCount(u16),
        FaultDetectionCounter(u8),
    }

    impl WireFormat for TestDTCExtData {
        fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
            let id = TestDTCExtDataRecordNumber::option_from_reader(reader)?;
            match id {
                Some(TestDTCExtDataRecordNumber::WarmUpCycleCount) => {
                    let count = reader.read_u16::<byteorder::BigEndian>()?;
                    Ok(Some(TestDTCExtData::WarmUpCycleCount(count)))
                }
                Some(TestDTCExtDataRecordNumber::FaultDetectionCounter) => {
                    let count = reader.read_u8()?;
                    Ok(Some(TestDTCExtData::FaultDetectionCounter(count)))
                }
                None => Ok(None),
            }
        }

        fn required_size(&self) -> usize {
            match self {
                TestDTCExtData::WarmUpCycleCount(_) => 3,
                TestDTCExtData::FaultDetectionCounter(_) => 2,
            }
        }

        fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
            match self {
                TestDTCExtData::WarmUpCycleCount(count) => {
                    writer.write_u8(TestDTCExtDataRecordNumber::WarmUpCycleCount as u8)?;
                    writer.write_u16::<byteorder::BigEndian>(*count)?;
                }
                TestDTCExtData::FaultDetectionCounter(count) => {
                    writer.write_u8(TestDTCExtDataRecordNumber::FaultDetectionCounter as u8)?;
                    writer.write_u8(*count)?;
                }
            }
            Ok(self.required_size())
        }
    }

    impl IterableWireFormat for TestDTCExtData {}

    #[test]
    fn ext_data_list() {
        // skip formatting
        #[rustfmt::skip]
        let bytes = [
            0x06, // subfunction
            // First DTC record
            0x12, 0x34, 0x56, // DTC Mask
            0x24, //Status
            0x04, // "WarmUpCycleCount"
            //Ext data
            0xBE, 0xEF,
            0x05, // "FaultDetectionCounter"
            0x10,

        ];
        let mut reader = &bytes[..];
        let response: ReadDTCInfoResponse<TestDTCExtData> =
            ReadDTCInfoResponse::from_reader(&mut reader).unwrap();

        // write
        let mut writer = Vec::new();
        let written = response.to_writer(&mut writer).unwrap();
        assert_eq!(writer, bytes, "Written: \n{:02X?}\n{:02X?}", writer, bytes);
        assert_eq!(written, bytes.len(), "Written: \n{:?}\n{:?}", writer, bytes);
        assert_eq!(written, response.required_size());
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::DTCStatusMask;

    #[test]
    fn test_read_dtc_information_request() {
        let bytes = [0x01, 0x01];
        let mut reader = &bytes[..];
        let mut writer = Vec::new();
        ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTCStoredData_ByRecordNumber(
            DTCStoredDataRecordNumber::new(5).unwrap(),
        ))
        .to_writer(&mut writer)
        .unwrap();
        let request = ReadDTCInfoRequest::option_from_reader(&mut reader)
            .unwrap()
            .unwrap();
        assert_eq!(
            request,
            ReadDTCInfoRequest {
                dtc_subfunction: ReadDTCInfoSubFunction::ReportNumberOfDTC_ByStatusMask(
                    DTCStatusMask::TestFailed
                )
            }
        );
    }

    #[test]
    fn test_read_dtc_information_subfunction() {
        let mut writer = Vec::new();
        let b = ReadDTCInfoSubFunction::ReportDTCWithPermanentStatus;
        b.to_writer(&mut writer).unwrap();

        assert_eq!(writer, vec![0x15]);

        for id in 0x01..=0x07 {
            let mut writer = Vec::new();
            let func = match id {
                0x01 => ReadDTCInfoSubFunction::ReportNumberOfDTC_ByStatusMask(
                    DTCStatusMask::TestFailed,
                ),
                0x02 => ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(
                    DTCStatusMask::WarningIndicatorRequested,
                ),
                0x03 => ReadDTCInfoSubFunction::ReportDTCSnapshotIdentification,
                0x04 => ReadDTCInfoSubFunction::ReportDTCSnapshotRecord_ByDTCNumber(
                    DTCRecord::new(0x01, 0x02, 0x03),
                    DTCSnapshotRecordNumber::new(0x04),
                ),
                0x05 => ReadDTCInfoSubFunction::ReportDTCStoredData_ByRecordNumber(
                    DTCStoredDataRecordNumber::new(0x20).unwrap(),
                ),
                0x06 => ReadDTCInfoSubFunction::ReportDTCExtDataRecord_ByDTCNumber(
                    DTCRecord::new(0x01, 0x02, 0x03),
                    DTCExtDataRecordNumber::new(0x04),
                ),
                0x07 => ReadDTCInfoSubFunction::ReportNumberOfDTC_BySeverityMaskRecord(
                    DTCSeverityMask::DTCClass_4,
                    DTCStatusMask::TestFailed,
                ),
                _ => unreachable!("Invalid loop value"),
            };
            let written = func.to_writer(&mut writer).unwrap();
            assert_eq!(written, func.required_size());
        }
    }
}
