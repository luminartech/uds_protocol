//! ReadDTCInformation (0x19) request and response service implementation
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    DTCExtDataRecordNumber, DTCMaskRecord, DTCSeverityMask, DTCSnapshotRecordNumber, DTCStatusMask,
    DTCStoredDataRecordNumber, FunctionalGroupIdentifier, UserDefDTCSnapshotRecordNumber,
};
use crate::{Error, SingleValueWireFormat, WireFormat};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

/// Used to address the respective user-defined DTC memory when retrieving DTCs
type MemorySelection = u8;
/// Have to reference SAE J1979-DA for the corresponding DTC readiness groups and the [FunctionalGroupIdentifier]s
/// This RGID depends on the functional group
type DTCReadinessGroupIdentifier = u8; // RGID

/// Subfunctions for the ReadDTCInformation service
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReadDTCInfoSubFunction {
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
    ///
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
    /// Parameter: DTCExtDataRecordNumber(1) (0xFF for all records)
    /// Parameter: MemorySelection(1)
    ///
    /// 0x19
    ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(
        DTCMaskRecord,
        DTCExtDataRecordNumber,
        MemorySelection,
    ),

    /// Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x1A
    ReportSupportedDTCExtDataRecord(DTCExtDataRecordNumber),

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
                DTCMaskRecord::from_reader(reader)?,
                DTCSnapshotRecordNumber::from_reader(reader)?,
            ),
            0x05 => Self::ReportDTCStoredData_ByRecordNumber(
                DTCStoredDataRecordNumber::from_reader(reader)?,
            ),
            // 0xFF for all records, 0xFE for all OBD records
            0x06 => Self::ReportDTCExtDataRecord_ByDTCNumber(
                DTCMaskRecord::from_reader(reader)?,
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
            0x09 => Self::ReportSeverityInfoOfDTC(DTCMaskRecord::from_reader(reader)?),
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
                DTCMaskRecord::from_reader(reader)?,
                UserDefDTCSnapshotRecordNumber::from_reader(reader)?,
                reader.read_u8()?,
            ),
            0x19 => Self::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(
                DTCMaskRecord::from_reader(reader)?,
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

#[cfg(test)]
mod tests {
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
                    DTCMaskRecord::new(0x01, 0x02, 0x03),
                    DTCSnapshotRecordNumber::new(0x04).unwrap(),
                ),
                0x05 => ReadDTCInfoSubFunction::ReportDTCStoredData_ByRecordNumber(
                    DTCStoredDataRecordNumber::new(0x20).unwrap(),
                ),
                0x06 => ReadDTCInfoSubFunction::ReportDTCExtDataRecord_ByDTCNumber(
                    DTCMaskRecord::new(0x01, 0x02, 0x03),
                    DTCExtDataRecordNumber(0x04),
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
