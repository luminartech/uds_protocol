//! `ReadDTCInformation` (0x19) request and response service implementation

use crate::{
    DTCExtDataRecordNumber, DTCFormatIdentifier, DTCRecord, DTCSeverityMask,
    DTCSnapshotRecordNumber, DTCStatusMask, DTCStoredDataRecordNumber, Decode, Encode, Error,
    FunctionalGroupIdentifier,
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
    /// Create a new `ReadDTCInfoRequest`.
    #[must_use]
    pub fn new(dtc_subfunction: ReadDTCInfoSubFunction) -> Self {
        Self { dtc_subfunction }
    }
}

impl Encode for ReadDTCInfoRequest {
    fn encoded_size(&self) -> usize {
        self.dtc_subfunction.encoded_size()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        self.dtc_subfunction.encode(writer)
    }
}

impl<'a> Decode<'a> for ReadDTCInfoRequest {
    #[allow(clippy::too_many_lines)]
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        use ReadDTCInfoSubFunction as S;
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let sub = buf[0];
        let rest = &buf[1..];
        let (dtc_subfunction, rest) = match sub {
            0x01 => {
                let (m, r) = DTCStatusMask::decode(rest)?;
                (S::ReportNumberOfDTC_ByStatusMask(m), r)
            }
            0x02 => {
                let (m, r) = DTCStatusMask::decode(rest)?;
                (S::ReportDTC_ByStatusMask(m), r)
            }
            0x03 => (S::ReportDTCSnapshotIdentification, rest),
            0x04 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCSnapshotRecordNumber::decode(r)?;
                (S::ReportDTCSnapshotRecord_ByDTCNumber(rec, n), r)
            }
            0x05 => {
                let (n, r) = DTCStoredDataRecordNumber::decode(rest)?;
                (S::ReportDTCStoredData_ByRecordNumber(n), r)
            }
            0x06 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCExtDataRecordNumber::decode(r)?;
                (S::ReportDTCExtDataRecord_ByDTCNumber(rec, n), r)
            }
            0x07 => {
                let (s, r) = DTCSeverityMask::decode(rest)?;
                let (m, r) = DTCStatusMask::decode(r)?;
                (S::ReportNumberOfDTC_BySeverityMaskRecord(s, m), r)
            }
            0x08 => {
                let (s, r) = DTCSeverityMask::decode(rest)?;
                let (m, r) = DTCStatusMask::decode(r)?;
                (S::ReportDTC_BySeverityMaskRecord(s, m), r)
            }
            0x09 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                (S::ReportSeverityInfoOfDTC(rec), r)
            }
            0x0A => (S::ReportSupportedDTC, rest),
            0x0B => (S::ReportFirstTestFailedDTC, rest),
            0x0C => (S::ReportFirstConfirmedDTC, rest),
            0x0D => (S::ReportMostRecentTestFailedDTC, rest),
            0x0E => (S::ReportMostRecentConfirmedDTC, rest),
            0x14 => (S::ReportDTCFaultDetectionCounter, rest),
            0x15 => (S::ReportDTCWithPermanentStatus, rest),
            0x16 => {
                let (n, r) = DTCExtDataRecordNumber::decode(rest)?;
                (S::ReportDTCExtDataRecord_ByRecordNumber(n), r)
            }
            0x17 => {
                let (m, r) = DTCStatusMask::decode(rest)?;
                (S::ReportUserDefMemoryDTC_ByStatusMask(m), r)
            }
            0x18 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCSnapshotRecordNumber::decode(r)?;
                let (mem, r) = u8::decode(r)?;
                (
                    S::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(rec, n, mem),
                    r,
                )
            }
            0x19 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCExtDataRecordNumber::decode(r)?;
                let (mem, r) = u8::decode(r)?;
                (
                    S::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(rec, n, mem),
                    r,
                )
            }
            0x1A => {
                let (n, r) = DTCExtDataRecordNumber::decode(rest)?;
                (S::ReportSupportedDTCExtDataRecord(n), r)
            }
            0x42 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                let (m, r) = DTCStatusMask::decode(r)?;
                let (s, r) = DTCSeverityMask::decode(r)?;
                (S::ReportWWHOBDDTC_ByMaskRecord(g, m, s), r)
            }
            0x55 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                (S::ReportWWHOBDDTC_WithPermanentStatus(g), r)
            }
            0x56 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                let (rg, r) = u8::decode(r)?;
                (
                    S::ReportDTCInformation_ByDTCReadinessGroupIdentifier(g, rg),
                    r,
                )
            }
            other => (S::ISOSAEReserved(other), rest),
        };
        Ok((ReadDTCInfoRequest::new(dtc_subfunction), rest))
    }
}

#[cfg(test)]
mod read_dtc_info_request_encode_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_no_param_subfunction() {
        // 0x0A ReportSupportedDTC, no parameters.
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportSupportedDTC);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x0A]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_single_param_subfunction() {
        // 0x02 ReportDTC_ByStatusMask(mask). DTCStatusMask is 1 byte.
        let mask = DTCStatusMask::from(0xFF);
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(mask));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_multi_param_subfunction() {
        // 0x42 ReportWWHOBDDTC_ByMaskRecord(group, status, severity).
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportWWHOBDDTC_ByMaskRecord(
            FunctionalGroupIdentifier::EmissionsSystemGroup,
            DTCStatusMask::from(0x08),
            DTCSeverityMask::CheckImmediately,
        ));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x42, 0x33, 0x08, 0b1000_0000]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_reserved_subfunction() {
        // ISOSAEReserved carries the sub-function byte itself, no params.
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ISOSAEReserved(0x57));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x57]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn read_dtc_info_request_roundtrips() {
        use crate::Decode;
        // Encode into a scratch buffer (oracle), then decode_exact and assert round-trip fidelity.
        let cases = [
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportSupportedDTC),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(
                DTCStatusMask::from(0xFF),
            )),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportWWHOBDDTC_ByMaskRecord(
                FunctionalGroupIdentifier::EmissionsSystemGroup,
                DTCStatusMask::from(0x08),
                DTCSeverityMask::CheckImmediately,
            )),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ISOSAEReserved(0x57)),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTCSnapshotRecord_ByDTCNumber(
                DTCRecord::new(0x12, 0x34, 0x56),
                DTCSnapshotRecordNumber::new(0x01),
            )),
        ];
        for req in cases {
            let mut buf = [0u8; 16];
            let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
            let decoded = <ReadDTCInfoRequest as Decode>::decode_exact(&buf[..written]).unwrap();
            assert_eq!(decoded, req);
        }
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

impl Encode for ReadDTCInfoSubFunction {
    fn encoded_size(&self) -> usize {
        use ReadDTCInfoSubFunction as S;
        1 + match self {
            S::ReportNumberOfDTC_ByStatusMask(m)
            | S::ReportDTC_ByStatusMask(m)
            | S::ReportUserDefMemoryDTC_ByStatusMask(m) => m.encoded_size(),
            S::ReportDTCSnapshotRecord_ByDTCNumber(r, n) => r.encoded_size() + n.encoded_size(),
            S::ReportDTCStoredData_ByRecordNumber(n) => n.encoded_size(),
            S::ReportDTCExtDataRecord_ByDTCNumber(r, n) => r.encoded_size() + n.encoded_size(),
            S::ReportNumberOfDTC_BySeverityMaskRecord(s, m)
            | S::ReportDTC_BySeverityMaskRecord(s, m) => s.encoded_size() + m.encoded_size(),
            S::ReportSeverityInfoOfDTC(r) => r.encoded_size(),
            S::ReportDTCExtDataRecord_ByRecordNumber(n) | S::ReportSupportedDTCExtDataRecord(n) => {
                n.encoded_size()
            }
            S::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(r, n, mem) => {
                r.encoded_size() + n.encoded_size() + mem.encoded_size()
            }
            S::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(r, n, mem) => {
                r.encoded_size() + n.encoded_size() + mem.encoded_size()
            }
            S::ReportWWHOBDDTC_ByMaskRecord(g, m, s) => {
                g.encoded_size() + m.encoded_size() + s.encoded_size()
            }
            S::ReportWWHOBDDTC_WithPermanentStatus(g) => g.encoded_size(),
            S::ReportDTCInformation_ByDTCReadinessGroupIdentifier(g, rg) => {
                g.encoded_size() + rg.encoded_size()
            }
            S::ReportDTCSnapshotIdentification
            | S::ReportSupportedDTC
            | S::ReportFirstTestFailedDTC
            | S::ReportFirstConfirmedDTC
            | S::ReportMostRecentTestFailedDTC
            | S::ReportMostRecentConfirmedDTC
            | S::ReportDTCFaultDetectionCounter
            | S::ReportDTCWithPermanentStatus
            | S::ISOSAEReserved(_) => 0,
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        use ReadDTCInfoSubFunction as S;
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        match self {
            S::ReportNumberOfDTC_ByStatusMask(m)
            | S::ReportDTC_ByStatusMask(m)
            | S::ReportUserDefMemoryDTC_ByStatusMask(m) => {
                m.encode(writer)?;
            }
            S::ReportDTCSnapshotRecord_ByDTCNumber(r, n) => {
                r.encode(writer)?;
                n.encode(writer)?;
            }
            S::ReportDTCStoredData_ByRecordNumber(n) => {
                n.encode(writer)?;
            }
            S::ReportDTCExtDataRecord_ByDTCNumber(r, n) => {
                r.encode(writer)?;
                n.encode(writer)?;
            }
            S::ReportNumberOfDTC_BySeverityMaskRecord(s, m)
            | S::ReportDTC_BySeverityMaskRecord(s, m) => {
                s.encode(writer)?;
                m.encode(writer)?;
            }
            S::ReportSeverityInfoOfDTC(r) => {
                r.encode(writer)?;
            }
            S::ReportDTCExtDataRecord_ByRecordNumber(n) | S::ReportSupportedDTCExtDataRecord(n) => {
                n.encode(writer)?;
            }
            S::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(r, n, mem) => {
                r.encode(writer)?;
                n.encode(writer)?;
                mem.encode(writer)?;
            }
            S::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(r, n, mem) => {
                r.encode(writer)?;
                n.encode(writer)?;
                mem.encode(writer)?;
            }
            S::ReportWWHOBDDTC_ByMaskRecord(g, m, s) => {
                g.encode(writer)?;
                m.encode(writer)?;
                s.encode(writer)?;
            }
            S::ReportWWHOBDDTC_WithPermanentStatus(g) => {
                g.encode(writer)?;
            }
            S::ReportDTCInformation_ByDTCReadinessGroupIdentifier(g, rg) => {
                g.encode(writer)?;
                rg.encode(writer)?;
            }
            S::ReportDTCSnapshotIdentification
            | S::ReportSupportedDTC
            | S::ReportFirstTestFailedDTC
            | S::ReportFirstConfirmedDTC
            | S::ReportMostRecentTestFailedDTC
            | S::ReportMostRecentConfirmedDTC
            | S::ReportDTCFaultDetectionCounter
            | S::ReportDTCWithPermanentStatus
            | S::ISOSAEReserved(_) => {}
        }
        Ok(self.encoded_size())
    }
}

/// Same representation as [`DTCStatusMask`] but with the bits 'on' representing the DTC status supported by the server
/// IE if the server doesn't support [`DTCStatusMask::WarningIndicatorRequested`] then the bit for that status will be 'off'
/// and all other bits will be 'on'
type DTCStatusAvailabilityMask = DTCStatusMask;

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
    pub fn collect_all(self) -> Result<alloc::vec::Vec<DTCFaultDetectionCounterRecord>, Error> {
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
        let dtc_record = DTCRecord::new(self.remaining[0], self.remaining[1], self.remaining[2]);
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

/// Zero-copy parsed response for `ReadDTCInformation` (0x19).
///
/// Stores raw bytes for record collections and provides lazy iterators
/// that parse on demand without allocation.
///
/// # Coverage
///
/// This enum models the sub-functions the library currently parses: `0x01`/`0x07`
/// (number of DTCs), `0x02`/`0x0A`–`0x0E`/`0x15` (DTC + status lists), `0x14` (fault
/// detection counters), `0x08`/`0x09` (DTC severity lists), and `0x42` (WWH-OBD by mask).
/// `ReadDTCInformation` defines further sub-functions that are **not yet modeled**;
/// [`decode`](Self::decode) returns [`Error::InvalidDtcSubfunctionType`] for those. See the
/// support table in the crate README. This is the "Partial" coverage noted there, not a bug.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ReadDTCInfoResponse<'a> {
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
        /// Raw `DTCAndSeverityRecord` bytes (6 bytes each: severity + DTC functional unit +
        /// 3-byte DTC + status). These differ from the 5-byte WWH-OBD records, so
        /// [`DtcSeverityAndStatusIter`] does **not** apply here; no severity-list iterator is
        /// wired yet, so parse these bytes caller-side until one is added.
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

impl<'a> ReadDTCInfoResponse<'a> {
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

impl<'a> Decode<'a> for ReadDTCInfoResponse<'a> {
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
            0x14 => Ok((Self::DTCFaultDetectionCounterList { raw_records: buf }, &[])),
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

impl Encode for ReadDTCInfoResponse<'_> {
    fn encoded_size(&self) -> usize {
        match self {
            Self::NumberOfDTCs { .. } => 4,
            Self::DTCList { raw_records, .. } | Self::DTCSeverityList { raw_records, .. } => {
                2 + raw_records.len()
            }
            Self::DTCFaultDetectionCounterList { raw_records } => 1 + raw_records.len(),
            Self::WWHOBDDTCByMaskRecord { raw_records, .. } => 5 + raw_records.len(),
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        match self {
            Self::NumberOfDTCs {
                sub_function_id,
                status_availability_mask,
                count,
            } => {
                writer
                    .write_all(&[*sub_function_id, status_availability_mask.bits()])
                    .map_err(Error::io)?;
                writer.write_all(&count.to_be_bytes()).map_err(Error::io)?;
            }
            Self::DTCList {
                sub_function_id,
                status_availability_mask,
                raw_records,
            }
            | Self::DTCSeverityList {
                sub_function_id,
                status_availability_mask,
                raw_records,
            } => {
                writer
                    .write_all(&[*sub_function_id, status_availability_mask.bits()])
                    .map_err(Error::io)?;
                writer.write_all(raw_records).map_err(Error::io)?;
            }
            Self::DTCFaultDetectionCounterList { raw_records } => {
                writer.write_all(&[0x14]).map_err(Error::io)?;
                writer.write_all(raw_records).map_err(Error::io)?;
            }
            Self::WWHOBDDTCByMaskRecord {
                functional_group_identifier,
                status_availability_mask,
                severity_availability_mask,
                format_identifier,
                raw_records,
            } => {
                writer
                    .write_all(&[
                        0x42,
                        u8::from(*functional_group_identifier),
                        status_availability_mask.bits(),
                        severity_availability_mask.bits(),
                        u8::from(*format_identifier),
                    ])
                    .map_err(Error::io)?;
                writer.write_all(raw_records).map_err(Error::io)?;
            }
        }
        Ok(self.encoded_size())
    }
}
