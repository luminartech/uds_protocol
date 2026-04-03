//! `ReadDTCInformation` (0x19) request and response service implementation

use crate::{
    DTCExtDataRecordNumber, DTCFormatIdentifier, DTCRecord, DTCSeverityMask,
    DTCSnapshotRecordNumber,
    DTCStatusMask, DTCStoredDataRecordNumber, Decode, Error, FunctionalGroupIdentifier,
};

/// Used for non-emissions related servers
type DTCFaultDetectionCounter = u8;
/// Used to address the respective user-defined DTC memory when retrieving DTCs
type MemorySelection = u8;

/// Request for the server to report diagnostic trouble code information
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ReadDTCInfoRequest {
    /// The sub-function specifying what DTC information to report.
    pub dtc_subfunction: ReadDTCInfoSubFunction,
}

impl ReadDTCInfoRequest {
    pub(crate) fn new(dtc_subfunction: ReadDTCInfoSubFunction) -> Self {
        Self { dtc_subfunction }
    }
}

/// A DTC paired with its fault detection counter value
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DTCFaultDetectionCounterRecord {
    pub dtc_record: DTCRecord,
    pub dtc_fault_detection_counter: DTCFaultDetectionCounter,
}

/// Have to reference SAE J1979-DA for the corresponding DTC readiness groups and the [`FunctionalGroupIdentifier`]s
/// This RGID depends on the functional group
type DTCReadinessGroupIdentifier = u8; // RGID

/// Subfunctions for the `ReadDTCInformation` service
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadDTCInfoSubFunction {
    /// * Parameter: `DTCStatusMask`
    ///
    /// 0x01
    ReportNumberOfDTC_ByStatusMask(DTCStatusMask),
    /// * Parameter: `DTCStatusMask`
    ///
    /// 0x02
    ReportDTC_ByStatusMask(DTCStatusMask),

    /// 0x03
    ReportDTCSnapshotIdentification,

    /// Parameter: `DTCRecord` (3 bytes)
    /// Parameter DTCSnapshotRecordNumber(1)
    ///
    /// 0x04
    ReportDTCSnapshotRecord_ByDTCNumber(DTCRecord, DTCSnapshotRecordNumber),

    /// * Parameter: DTCStoredDataRecordNumber(1)
    ///
    /// 0x05
    ReportDTCStoredData_ByRecordNumber(DTCStoredDataRecordNumber),

    /// Parameter: `DTCRecord` (3 bytes)
    /// Parameter: DTCExtDataRecordNumber(1)
    ///
    /// 0x06
    ReportDTCExtDataRecord_ByDTCNumber(DTCRecord, DTCExtDataRecordNumber),

    /// * Parameter: DTCSeverityMaskRecord(2)
    ///     * `DTCSeverityMask`
    ///     * `DTCStatusMask`
    ///
    /// 0x07
    ReportNumberOfDTC_BySeverityMaskRecord(DTCSeverityMask, DTCStatusMask),
    /// 0x08
    ReportDTC_BySeverityMaskRecord(DTCSeverityMask, DTCStatusMask),

    /// Parameter: `DTCRecord` (3 bytes)
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

    /// * Parameter: `DTCStatusMask`
    ///
    /// 0x17
    ReportUserDefMemoryDTC_ByStatusMask(DTCStatusMask),

    // TODO: UserDef and MemorySelection might just need to be u8
    /// 0x18
    ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(
        DTCRecord,
        DTCSnapshotRecordNumber,
        MemorySelection,
    ),

    /// Parameter: `DTCRecord` (3 bytes)
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
    /// * Parameter: `DTCStatusMask`
    /// * Parameter: `DTCSeverityMask`
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
    /// Return the raw `u8` sub-function byte.
    #[must_use]
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

type NumberOfDTCs = u16;
/// Same representation as [`DTCStatusMask`] but with the bits 'on' representing the DTC status supported by the server
/// IE if the server doesn't support [`DTCStatusMask::WarningIndicatorRequested`] then the bit for that status will be 'off'
/// and all other bits will be 'on'
type DTCStatusAvailabilityMask = DTCStatusMask;

/// Subfunction ID for the response
type SubFunctionID = u8;

/// Response payloads can be shared among multiple request subfunctions

// ---------------------------------------------------------------------------
// no_std RX types with lazy iterators
// ---------------------------------------------------------------------------

/// Lazy iterator over `(DTCRecord, DTCStatusMask)` pairs from raw bytes.
///
/// Each pair is 4 bytes: 3 for the DTC record + 1 for the status mask.
#[derive(Clone, Debug)]
pub struct DtcAndStatusIter<'a> {
    remaining: &'a [u8],
}

impl<'a> DtcAndStatusIter<'a> {
    /// Create an iterator over `(DTCRecord, DTCStatusMask)` pairs.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Number of complete records available.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.remaining.len() / 4
    }

    /// Whether there are no records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }

    /// Collect all records into a `Vec`.
    ///
    /// # Errors
    /// Returns an error if the byte data contains a partial record.
    #[cfg(feature = "alloc")]
    pub fn collect_all(self) -> Result<alloc::vec::Vec<(DTCRecord, DTCStatusMask)>, Error> {
        self.collect()
    }
}

impl Iterator for DtcAndStatusIter<'_> {
    type Item = Result<(DTCRecord, DTCStatusMask), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        if self.remaining.len() < 4 {
            return Some(Err(Error::IncorrectMessageLengthOrInvalidFormat));
        }
        let record = DTCRecord::new(self.remaining[0], self.remaining[1], self.remaining[2]);
        let status = DTCStatusMask::from(self.remaining[3]);
        self.remaining = &self.remaining[4..];
        Some(Ok((record, status)))
    }
}

/// Lazy iterator over `DTCFaultDetectionCounterRecord` from raw bytes.
///
/// Each record is 4 bytes: 3 for the DTC record + 1 for the fault detection counter.
#[derive(Clone, Debug)]
pub struct DtcFaultDetectionIter<'a> {
    remaining: &'a [u8],
}

impl<'a> DtcFaultDetectionIter<'a> {
    /// Create an iterator over `DTCFaultDetectionCounterRecord` values.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Collect all records into a `Vec`.
    ///
    /// # Errors
    /// Returns an error if the byte data contains a partial record.
    #[cfg(feature = "alloc")]
    pub fn collect_all(
        self,
    ) -> Result<alloc::vec::Vec<DTCFaultDetectionCounterRecord>, Error> {
        self.collect()
    }
}

impl Iterator for DtcFaultDetectionIter<'_> {
    type Item = Result<DTCFaultDetectionCounterRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        if self.remaining.len() < 4 {
            return Some(Err(Error::IncorrectMessageLengthOrInvalidFormat));
        }
        let dtc_record =
            DTCRecord::new(self.remaining[0], self.remaining[1], self.remaining[2]);
        let dtc_fault_detection_counter = self.remaining[3];
        self.remaining = &self.remaining[4..];
        Some(Ok(DTCFaultDetectionCounterRecord {
            dtc_record,
            dtc_fault_detection_counter,
        }))
    }
}

/// Lazy iterator over `(DTCSeverityMask, DTCRecord, DTCStatusMask)` triples from raw bytes.
///
/// Each triple is 5 bytes: 1 severity + 3 DTC record + 1 status mask.
#[derive(Clone, Debug)]
pub struct DtcSeverityAndStatusIter<'a> {
    remaining: &'a [u8],
}

impl<'a> DtcSeverityAndStatusIter<'a> {
    /// Create an iterator over severity/DTC/status triples.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Collect all triples into a `Vec`.
    ///
    /// # Errors
    /// Returns an error if the byte data contains a partial record.
    #[cfg(feature = "alloc")]
    pub fn collect_all(
        self,
    ) -> Result<alloc::vec::Vec<(DTCSeverityMask, DTCRecord, DTCStatusMask)>, Error> {
        self.collect()
    }
}

impl Iterator for DtcSeverityAndStatusIter<'_> {
    type Item = Result<(DTCSeverityMask, DTCRecord, DTCStatusMask), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        if self.remaining.len() < 5 {
            return Some(Err(Error::IncorrectMessageLengthOrInvalidFormat));
        }
        let severity = DTCSeverityMask::from(self.remaining[0]);
        let record = DTCRecord::new(self.remaining[1], self.remaining[2], self.remaining[3]);
        let status = DTCStatusMask::from(self.remaining[4]);
        self.remaining = &self.remaining[5..];
        Some(Ok((severity, record, status)))
    }
}

/// Zero-copy RX response for `ReadDTCInformation` (0x19).
///
/// Stores raw bytes for record collections and provides lazy iterators
/// that parse on demand without allocation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ReadDTCInfoResponseRx<'a> {
    /// Sub-functions 0x01, 0x07: count of DTCs matching a mask.
    NumberOfDTCs {
        /// Sub-function byte echo.
        sub_function_id: u8,
        /// DTC status availability mask.
        status_availability_mask: DTCStatusAvailabilityMask,
        /// Number of matching DTCs.
        count: u16,
    },
    /// Sub-functions 0x02, 0x0A-0x0E, 0x15: list of `(DTCRecord, DTCStatusMask)` pairs.
    DTCList {
        /// Sub-function byte echo.
        sub_function_id: u8,
        /// DTC status availability mask.
        status_availability_mask: DTCStatusAvailabilityMask,
        /// Raw record bytes — use [`DtcAndStatusIter`] to iterate.
        raw_records: &'a [u8],
    },
    /// Sub-function 0x14: list of DTC fault detection counter records.
    DTCFaultDetectionCounterList {
        /// Raw record bytes — use [`DtcFaultDetectionIter`] to iterate.
        raw_records: &'a [u8],
    },
    /// Sub-functions 0x08, 0x09: list of DTC severity records.
    DTCSeverityList {
        /// Sub-function byte echo.
        sub_function_id: u8,
        /// DTC status availability mask.
        status_availability_mask: DTCStatusAvailabilityMask,
        /// Raw record bytes (6 bytes per record) — use [`DTCSeverityRecord`] iteration.
        raw_records: &'a [u8],
    },
    /// Sub-function 0x42: WWH-OBD DTC by mask with severity info.
    WWHOBDDTCByMaskRecord {
        /// Functional group identifier echo.
        functional_group_identifier: FunctionalGroupIdentifier,
        /// DTC status availability mask.
        status_availability_mask: DTCStatusAvailabilityMask,
        /// Severity availability mask.
        severity_availability_mask: DTCSeverityMask,
        /// DTC format identifier.
        format_identifier: DTCFormatIdentifier,
        /// Raw record bytes (5 bytes per record) — use [`DtcSeverityAndStatusIter`].
        raw_records: &'a [u8],
    },
}

impl<'a> ReadDTCInfoResponseRx<'a> {
    /// Iterate `(DTCRecord, DTCStatusMask)` pairs for `DTCList` variants.
    ///
    /// Returns `None` if this is not a `DTCList` variant.
    #[must_use]
    pub fn dtc_and_status_iter(&self) -> Option<DtcAndStatusIter<'a>> {
        match self {
            Self::DTCList { raw_records, .. } => Some(DtcAndStatusIter::new(raw_records)),
            _ => None,
        }
    }

    /// Iterate fault detection counter records for the `DTCFaultDetectionCounterList` variant.
    ///
    /// Returns `None` if this is not that variant.
    #[must_use]
    pub fn fault_detection_iter(&self) -> Option<DtcFaultDetectionIter<'a>> {
        match self {
            Self::DTCFaultDetectionCounterList { raw_records } => {
                Some(DtcFaultDetectionIter::new(raw_records))
            }
            _ => None,
        }
    }

    /// Iterate severity/DTC/status triples for WWH-OBD variants.
    ///
    /// Returns `None` if this is not a severity variant.
    #[must_use]
    pub fn severity_and_status_iter(&self) -> Option<DtcSeverityAndStatusIter<'a>> {
        match self {
            Self::WWHOBDDTCByMaskRecord { raw_records, .. } => {
                Some(DtcSeverityAndStatusIter::new(raw_records))
            }
            _ => None,
        }
    }
}

impl<'a> Decode<'a> for ReadDTCInfoResponseRx<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let subfunction_id = buf[0];
        let buf = &buf[1..];

        match subfunction_id {
            0x01 | 0x07 => {
                if buf.len() < 3 {
                    return Err(Error::InsufficientData(4));
                }
                let status_availability_mask = DTCStatusAvailabilityMask::from(buf[0]);
                let count = u16::from_be_bytes([buf[1], buf[2]]);
                Ok((
                    Self::NumberOfDTCs {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        count,
                    },
                    &buf[3..],
                ))
            }
            0x02 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x15 => {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(2));
                }
                let status_availability_mask = DTCStatusAvailabilityMask::from(buf[0]);
                Ok((
                    Self::DTCList {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        raw_records: &buf[1..],
                    },
                    &[],
                ))
            }
            0x14 => Ok((
                Self::DTCFaultDetectionCounterList {
                    raw_records: buf,
                },
                &[],
            )),
            0x08 | 0x09 => {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(2));
                }
                let status_availability_mask = DTCStatusAvailabilityMask::from(buf[0]);
                Ok((
                    Self::DTCSeverityList {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        raw_records: &buf[1..],
                    },
                    &[],
                ))
            }
            0x42 => {
                if buf.len() < 4 {
                    return Err(Error::InsufficientData(5));
                }
                let functional_group_identifier = FunctionalGroupIdentifier::from(buf[0]);
                let status_availability_mask = DTCStatusAvailabilityMask::from(buf[1]);
                let severity_availability_mask = DTCSeverityMask::from(buf[2]);
                let format_identifier = DTCFormatIdentifier::from(buf[3]);
                Ok((
                    Self::WWHOBDDTCByMaskRecord {
                        functional_group_identifier,
                        status_availability_mask,
                        severity_availability_mask,
                        format_identifier,
                        raw_records: &buf[4..],
                    },
                    &[],
                ))
            }
            _ => Err(Error::InvalidDtcSubfunctionType(subfunction_id)),
        }
    }
}

