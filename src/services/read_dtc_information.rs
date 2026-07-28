//! `ReadDTCInformation` (0x19) request and response service implementation

use automotive_wire_codec::{read_u8, write_all, write_u8, write_u16_be};

use crate::{
    Decode, DtcExtDataRecordNumber, DtcFormatIdentifier, DtcRecord, DtcSeverityMask,
    DtcSnapshotRecordNumber, DtcStatusMask, DtcStoredDataRecordNumber, Encode, Error,
    FunctionalGroupIdentifier, Incomplete, NegativeResponseCode,
};

const READ_DTC_INFO_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 3] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::RequestOutOfRange,
];

/// Request for the server to report diagnostic trouble code information
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ReadDtcInfoRequest {
    /// The sub-function specifying what DTC information to report.
    pub dtc_subfunction: ReadDtcInfoSubFunction,
}

impl ReadDtcInfoRequest {
    /// Create a new `ReadDtcInfoRequest`.
    #[must_use]
    pub const fn new(dtc_subfunction: ReadDtcInfoSubFunction) -> Self {
        Self { dtc_subfunction }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DTC_INFO_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ReadDtcInfoRequest {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        self.dtc_subfunction.encode(writer)
    }
}

impl<'a> Decode<'a> for ReadDtcInfoRequest {
    type Error = crate::Error;

    #[allow(clippy::too_many_lines)]
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        use ReadDtcInfoSubFunction as S;
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let sub = buf[0];
        let rest = &buf[1..];
        let (dtc_subfunction, rest) = match sub {
            0x01 => {
                let (m, r) = DtcStatusMask::decode(rest)?;
                (S::ReportNumberOfDtcByStatusMask(m), r)
            }
            0x02 => {
                let (m, r) = DtcStatusMask::decode(rest)?;
                (S::ReportDtcByStatusMask(m), r)
            }
            0x03 => (S::ReportDtcSnapshotIdentification, rest),
            0x04 => {
                let (rec, r) = DtcRecord::decode(rest)?;
                let (n, r) = DtcSnapshotRecordNumber::decode(r)?;
                (S::ReportDtcSnapshotRecordByDtcNumber(rec, n), r)
            }
            0x05 => {
                let (n, r) = DtcStoredDataRecordNumber::decode(rest)?;
                (S::ReportDtcStoredDataByRecordNumber(n), r)
            }
            0x06 => {
                let (rec, r) = DtcRecord::decode(rest)?;
                let (n, r) = DtcExtDataRecordNumber::decode(r)?;
                (S::ReportDtcExtDataRecordByDtcNumber(rec, n), r)
            }
            0x07 => {
                let (s, r) = DtcSeverityMask::decode(rest)?;
                let (m, r) = DtcStatusMask::decode(r)?;
                (S::ReportNumberOfDtcBySeverityMaskRecord(s, m), r)
            }
            0x08 => {
                let (s, r) = DtcSeverityMask::decode(rest)?;
                let (m, r) = DtcStatusMask::decode(r)?;
                (S::ReportDtcBySeverityMaskRecord(s, m), r)
            }
            0x09 => {
                let (rec, r) = DtcRecord::decode(rest)?;
                (S::ReportSeverityInfoOfDtc(rec), r)
            }
            0x0A => (S::ReportSupportedDtc, rest),
            0x0B => (S::ReportFirstTestFailedDtc, rest),
            0x0C => (S::ReportFirstConfirmedDtc, rest),
            0x0D => (S::ReportMostRecentTestFailedDtc, rest),
            0x0E => (S::ReportMostRecentConfirmedDtc, rest),
            0x14 => (S::ReportDtcFaultDetectionCounter, rest),
            0x15 => (S::ReportDtcWithPermanentStatus, rest),
            0x16 => {
                let (n, r) = DtcExtDataRecordNumber::decode(rest)?;
                (S::ReportDtcExtDataRecordByRecordNumber(n), r)
            }
            0x17 => {
                let (m, r) = DtcStatusMask::decode(rest)?;
                (S::ReportUserDefMemoryDtcByStatusMask(m), r)
            }
            0x18 => {
                let (rec, r) = DtcRecord::decode(rest)?;
                let (n, r) = DtcSnapshotRecordNumber::decode(r)?;
                let (mem, r) = read_u8(r)?;
                (
                    S::ReportUserDefMemoryDtcSnapshotRecordByDtcNumber(rec, n, mem),
                    r,
                )
            }
            0x19 => {
                let (rec, r) = DtcRecord::decode(rest)?;
                let (n, r) = DtcExtDataRecordNumber::decode(r)?;
                let (mem, r) = read_u8(r)?;
                (
                    S::ReportUserDefMemoryDtcExtDataRecordByDtcNumber(rec, n, mem),
                    r,
                )
            }
            0x1A => {
                let (n, r) = DtcExtDataRecordNumber::decode(rest)?;
                (S::ReportSupportedDtcExtDataRecord(n), r)
            }
            0x42 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                let (m, r) = DtcStatusMask::decode(r)?;
                let (s, r) = DtcSeverityMask::decode(r)?;
                (S::ReportWwhObdDtcByMaskRecord(g, m, s), r)
            }
            0x55 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                (S::ReportWwhObdDtcWithPermanentStatus(g), r)
            }
            0x56 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                let (rg, r) = read_u8(r)?;
                (
                    S::ReportDtcInformationByDtcReadinessGroupIdentifier(g, rg),
                    r,
                )
            }
            other => (S::IsoSaeReserved(other), rest),
        };
        Ok((ReadDtcInfoRequest::new(dtc_subfunction), rest))
    }
}

#[cfg(test)]
mod read_dtc_info_request_encode_tests {
    use super::*;
    use crate::{NegativeResponseCode, test_util::assert_encode_size_agrees};

    #[test]
    fn encode_no_param_subfunction() {
        // 0x0A ReportSupportedDtc, no parameters.
        let req = ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportSupportedDtc);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x0A]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_single_param_subfunction() {
        // 0x02 ReportDtcByStatusMask(mask). DtcStatusMask is 1 byte.
        let mask = DtcStatusMask::from(0xFF);
        let req = ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportDtcByStatusMask(mask));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_multi_param_subfunction() {
        // 0x42 ReportWwhObdDtcByMaskRecord(group, status, severity).
        let req = ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportWwhObdDtcByMaskRecord(
            FunctionalGroupIdentifier::EmissionsSystemGroup,
            DtcStatusMask::from(0x08),
            DtcSeverityMask::CheckImmediately,
        ));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x42, 0x33, 0x08, 0b1000_0000]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_reserved_subfunction() {
        // IsoSaeReserved carries the sub-function byte itself, no params.
        let req = ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::IsoSaeReserved(0x57));
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
            ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportSupportedDtc),
            ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportDtcByStatusMask(
                DtcStatusMask::from(0xFF),
            )),
            ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportWwhObdDtcByMaskRecord(
                FunctionalGroupIdentifier::EmissionsSystemGroup,
                DtcStatusMask::from(0x08),
                DtcSeverityMask::CheckImmediately,
            )),
            ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::IsoSaeReserved(0x57)),
            ReadDtcInfoRequest::new(ReadDtcInfoSubFunction::ReportDtcSnapshotRecordByDtcNumber(
                DtcRecord::new(0x12, 0x34, 0x56),
                DtcSnapshotRecordNumber::new(0x01),
            )),
        ];
        for req in cases {
            let mut buf = [0u8; 16];
            let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
            let decoded = <ReadDtcInfoRequest as Decode>::decode_exact(&buf[..written]).unwrap();
            assert_eq!(decoded, req);
        }
    }

    #[test]
    fn exposes_allowed_nack_codes() {
        assert!(!ReadDtcInfoRequest::allowed_nack_codes().is_empty());
        assert!(
            ReadDtcInfoRequest::allowed_nack_codes()
                .contains(&NegativeResponseCode::RequestOutOfRange)
        );
    }
}

/// A DTC paired with its fault detection counter value
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DtcFaultDetectionCounterRecord {
    /// The DTC this counter belongs to.
    pub dtc_record: DtcRecord,
    /// Fault detection counter, used for non-emissions related servers.
    pub dtc_fault_detection_counter: u8,
}

/// Subfunctions for the `ReadDTCInformation` service
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadDtcInfoSubFunction {
    /// * Parameter: `DtcStatusMask`
    ///
    /// 0x01
    ReportNumberOfDtcByStatusMask(DtcStatusMask),
    /// * Parameter: `DtcStatusMask`
    ///
    /// 0x02
    ReportDtcByStatusMask(DtcStatusMask),

    /// 0x03
    ReportDtcSnapshotIdentification,

    /// Parameter: `DtcRecord` (3 bytes)
    /// Parameter DtcSnapshotRecordNumber(1)
    ///
    /// 0x04
    ReportDtcSnapshotRecordByDtcNumber(DtcRecord, DtcSnapshotRecordNumber),

    /// * Parameter: DtcStoredDataRecordNumber(1)
    ///
    /// 0x05
    ReportDtcStoredDataByRecordNumber(DtcStoredDataRecordNumber),

    /// Parameter: `DtcRecord` (3 bytes)
    /// Parameter: DtcExtDataRecordNumber(1)
    ///
    /// 0x06
    ReportDtcExtDataRecordByDtcNumber(DtcRecord, DtcExtDataRecordNumber),

    /// * Parameter: DTCSeverityMaskRecord(2)
    ///     * `DtcSeverityMask`
    ///     * `DtcStatusMask`
    ///
    /// 0x07
    ReportNumberOfDtcBySeverityMaskRecord(DtcSeverityMask, DtcStatusMask),
    /// 0x08
    ReportDtcBySeverityMaskRecord(DtcSeverityMask, DtcStatusMask),

    /// Parameter: `DtcRecord` (3 bytes)
    ///
    /// 0x09
    ReportSeverityInfoOfDtc(DtcRecord),

    /// 0x0A
    ReportSupportedDtc,
    /// 0x0B
    ReportFirstTestFailedDtc,
    /// 0x0C
    ReportFirstConfirmedDtc,
    /// 0x0D
    ReportMostRecentTestFailedDtc,
    /// 0x0E
    ReportMostRecentConfirmedDtc,
    /// 0x14
    ReportDtcFaultDetectionCounter,
    /// 0x15
    ReportDtcWithPermanentStatus,

    /// * Parameter: DtcExtDataRecordNumber(1)
    ///
    /// 0x16
    ReportDtcExtDataRecordByRecordNumber(DtcExtDataRecordNumber),

    /// * Parameter: `DtcStatusMask`
    ///
    /// 0x17
    ReportUserDefMemoryDtcByStatusMask(DtcStatusMask),

    /// Parameter: `DtcRecord` (3 bytes)
    /// Parameter: `DtcSnapshotRecordNumber`(1)
    /// Parameter: `memorySelection`(1) — addresses the user-defined DTC memory to read from
    ///
    /// 0x18
    ReportUserDefMemoryDtcSnapshotRecordByDtcNumber(DtcRecord, DtcSnapshotRecordNumber, u8),

    /// Parameter: `DtcRecord` (3 bytes)
    /// Parameter: `DtcExtDataRecordNumber`(1) (0xFF for all records)
    /// Parameter: `memorySelection`(1) — addresses the user-defined DTC memory to read from
    ///
    /// 0x19
    ReportUserDefMemoryDtcExtDataRecordByDtcNumber(DtcRecord, DtcExtDataRecordNumber, u8),

    /// * Parameter: DtcExtDataRecordNumber(1)
    ///
    /// 0x1A
    ReportSupportedDtcExtDataRecord(DtcExtDataRecordNumber),

    /// * Parameter: FunctionalGroupIdentifier(1)
    /// * Parameter: `DtcStatusMask`
    /// * Parameter: `DtcSeverityMask`
    ///
    /// 0x42
    ReportWwhObdDtcByMaskRecord(FunctionalGroupIdentifier, DtcStatusMask, DtcSeverityMask),

    /// * Parameter: FunctionalGroupIdentifier(1)
    ///
    /// 0x55
    ReportWwhObdDtcWithPermanentStatus(FunctionalGroupIdentifier),

    /// * Parameter: `FunctionalGroupIdentifier`(1)
    /// * Parameter: `DTCReadinessGroupIdentifier` (RGID, 1 byte). The RGID depends on the
    ///   functional group; see SAE J1979-DA for the readiness groups that correspond to each
    ///   [`FunctionalGroupIdentifier`].
    ///
    /// 0x56
    ReportDtcInformationByDtcReadinessGroupIdentifier(FunctionalGroupIdentifier, u8),
    /// 0x42-0x54, 0x57-0x7F
    IsoSaeReserved(u8),
}

impl ReadDtcInfoSubFunction {
    /// Return the raw `u8` sub-function byte.
    #[must_use]
    pub const fn value(&self) -> u8 {
        match self {
            Self::ReportNumberOfDtcByStatusMask(_) => 0x01,
            Self::ReportDtcByStatusMask(_) => 0x02,
            Self::ReportDtcSnapshotIdentification => 0x03,
            Self::ReportDtcSnapshotRecordByDtcNumber(_, _) => 0x04,
            Self::ReportDtcStoredDataByRecordNumber(_) => 0x05,
            Self::ReportDtcExtDataRecordByDtcNumber(_, _) => 0x06,
            Self::ReportNumberOfDtcBySeverityMaskRecord(_, _) => 0x07,
            Self::ReportDtcBySeverityMaskRecord(_, _) => 0x08,
            Self::ReportSeverityInfoOfDtc(_) => 0x09,
            Self::ReportSupportedDtc => 0x0A,
            Self::ReportFirstTestFailedDtc => 0x0B,
            Self::ReportFirstConfirmedDtc => 0x0C,
            Self::ReportMostRecentTestFailedDtc => 0x0D,
            Self::ReportMostRecentConfirmedDtc => 0x0E,
            Self::ReportDtcFaultDetectionCounter => 0x14,
            Self::ReportDtcWithPermanentStatus => 0x15,
            Self::ReportDtcExtDataRecordByRecordNumber(_) => 0x16,
            Self::ReportUserDefMemoryDtcByStatusMask(_) => 0x17,
            Self::ReportUserDefMemoryDtcSnapshotRecordByDtcNumber(_, _, _) => 0x18,
            Self::ReportUserDefMemoryDtcExtDataRecordByDtcNumber(_, _, _) => 0x19,
            Self::ReportSupportedDtcExtDataRecord(_) => 0x1A,
            Self::ReportWwhObdDtcByMaskRecord(_, _, _) => 0x42,
            Self::ReportWwhObdDtcWithPermanentStatus(_) => 0x55,
            Self::ReportDtcInformationByDtcReadinessGroupIdentifier(_, _) => 0x56,
            Self::IsoSaeReserved(value) => *value,
        }
    }
}

impl Encode for ReadDtcInfoSubFunction {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        use ReadDtcInfoSubFunction as S;
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        let mut written = 1;
        match self {
            S::ReportNumberOfDtcByStatusMask(m)
            | S::ReportDtcByStatusMask(m)
            | S::ReportUserDefMemoryDtcByStatusMask(m) => {
                written += m.encode(writer)?;
            }
            S::ReportDtcSnapshotRecordByDtcNumber(r, n) => {
                written += r.encode(writer)?;
                written += n.encode(writer)?;
            }
            S::ReportDtcStoredDataByRecordNumber(n) => {
                written += n.encode(writer)?;
            }
            S::ReportDtcExtDataRecordByDtcNumber(r, n) => {
                written += r.encode(writer)?;
                written += n.encode(writer)?;
            }
            S::ReportNumberOfDtcBySeverityMaskRecord(s, m)
            | S::ReportDtcBySeverityMaskRecord(s, m) => {
                written += s.encode(writer)?;
                written += m.encode(writer)?;
            }
            S::ReportSeverityInfoOfDtc(r) => {
                written += r.encode(writer)?;
            }
            S::ReportDtcExtDataRecordByRecordNumber(n) | S::ReportSupportedDtcExtDataRecord(n) => {
                written += n.encode(writer)?;
            }
            S::ReportUserDefMemoryDtcSnapshotRecordByDtcNumber(r, n, mem) => {
                written += r.encode(writer)?;
                written += n.encode(writer)?;
                written += write_u8(writer, *mem).map_err(Error::io)?;
            }
            S::ReportUserDefMemoryDtcExtDataRecordByDtcNumber(r, n, mem) => {
                written += r.encode(writer)?;
                written += n.encode(writer)?;
                written += write_u8(writer, *mem).map_err(Error::io)?;
            }
            S::ReportWwhObdDtcByMaskRecord(g, m, s) => {
                written += g.encode(writer)?;
                written += m.encode(writer)?;
                written += s.encode(writer)?;
            }
            S::ReportWwhObdDtcWithPermanentStatus(g) => {
                written += g.encode(writer)?;
            }
            S::ReportDtcInformationByDtcReadinessGroupIdentifier(g, rg) => {
                written += g.encode(writer)?;
                written += write_u8(writer, *rg).map_err(Error::io)?;
            }
            S::ReportDtcSnapshotIdentification
            | S::ReportSupportedDtc
            | S::ReportFirstTestFailedDtc
            | S::ReportFirstConfirmedDtc
            | S::ReportMostRecentTestFailedDtc
            | S::ReportMostRecentConfirmedDtc
            | S::ReportDtcFaultDetectionCounter
            | S::ReportDtcWithPermanentStatus
            | S::IsoSaeReserved(_) => {}
        }
        Ok(written)
    }
}

// ---------------------------------------------------------------------------
// no_std RX types with lazy iterators
// ---------------------------------------------------------------------------

/// Lazy iterator over `(DtcRecord, DtcStatusMask)` pairs from raw bytes.
///
/// Each pair is 4 bytes: 3 for the DTC record + 1 for the status mask.
///
/// # Length
///
/// [`len`](DtcAndStatusIter::len) counts **complete records**; [`size_hint`](Iterator::size_hint) counts
/// **items yielded**, which is one greater when a partial record trails the buffer (that tail
/// surfaces as a single `Err`, after which the iterator is exhausted). The two therefore differ
/// on malformed input, which is why this deliberately does not implement `ExactSizeIterator` —
/// its `len()` would contradict the inherent one. It does implement
/// [`FusedIterator`](core::iter::FusedIterator).
#[derive(Clone, Debug)]
pub struct DtcAndStatusIter<'a> {
    remaining: &'a [u8],
}

impl<'a> DtcAndStatusIter<'a> {
    /// Create an iterator over `(DtcRecord, DtcStatusMask)` pairs.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Number of complete records available.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.remaining.len() / 4
    }

    /// Whether there are no complete records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Collect all records into a `Vec`.
    ///
    /// # Errors
    /// Returns an error if the byte data contains a partial record.
    #[cfg(feature = "alloc")]
    pub fn collect_all(self) -> Result<alloc::vec::Vec<(DtcRecord, DtcStatusMask)>, Error> {
        self.collect()
    }
}

impl Iterator for DtcAndStatusIter<'_> {
    type Item = Result<(DtcRecord, DtcStatusMask), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        if self.remaining.len() < 4 {
            // Consume the partial tail so the error is reported exactly once and the
            // iterator terminates. Returning without advancing would yield this error
            // forever.
            self.remaining = &[];
            return Some(Err(Error::IncorrectMessageLengthOrInvalidFormat));
        }
        let record = DtcRecord::new(self.remaining[0], self.remaining[1], self.remaining[2]);
        let status = DtcStatusMask::from(self.remaining[3]);
        self.remaining = &self.remaining[4..];
        Some(Ok((record, status)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // One item per complete record, plus one final `Err` if a partial tail remains.
        let n = self.remaining.len().div_ceil(4);
        (n, Some(n))
    }
}

impl core::iter::FusedIterator for DtcAndStatusIter<'_> {}

/// Lazy iterator over `DtcFaultDetectionCounterRecord` from raw bytes.
///
/// Each record is 4 bytes: 3 for the DTC record + 1 for the fault detection counter.
///
/// # Length
///
/// [`len`](DtcFaultDetectionIter::len) counts **complete records**; [`size_hint`](Iterator::size_hint) counts
/// **items yielded**, which is one greater when a partial record trails the buffer (that tail
/// surfaces as a single `Err`, after which the iterator is exhausted). The two therefore differ
/// on malformed input, which is why this deliberately does not implement `ExactSizeIterator` —
/// its `len()` would contradict the inherent one. It does implement
/// [`FusedIterator`](core::iter::FusedIterator).
#[derive(Clone, Debug)]
pub struct DtcFaultDetectionIter<'a> {
    remaining: &'a [u8],
}

impl<'a> DtcFaultDetectionIter<'a> {
    /// Create an iterator over `DtcFaultDetectionCounterRecord` values.
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    /// Number of complete records available.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.remaining.len() / 4
    }

    /// Whether there are no complete records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Collect all records into a `Vec`.
    ///
    /// # Errors
    /// Returns an error if the byte data contains a partial record.
    #[cfg(feature = "alloc")]
    pub fn collect_all(self) -> Result<alloc::vec::Vec<DtcFaultDetectionCounterRecord>, Error> {
        self.collect()
    }
}

impl Iterator for DtcFaultDetectionIter<'_> {
    type Item = Result<DtcFaultDetectionCounterRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        if self.remaining.len() < 4 {
            // See `DtcAndStatusIter::next`: consume the tail so this terminates.
            self.remaining = &[];
            return Some(Err(Error::IncorrectMessageLengthOrInvalidFormat));
        }
        let dtc_record = DtcRecord::new(self.remaining[0], self.remaining[1], self.remaining[2]);
        let dtc_fault_detection_counter = self.remaining[3];
        self.remaining = &self.remaining[4..];
        Some(Ok(DtcFaultDetectionCounterRecord {
            dtc_record,
            dtc_fault_detection_counter,
        }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining.len().div_ceil(4);
        (n, Some(n))
    }
}

impl core::iter::FusedIterator for DtcFaultDetectionIter<'_> {}

/// Lazy iterator over `(DtcSeverityMask, DtcRecord, DtcStatusMask)` triples from raw bytes.
///
/// Each triple is 5 bytes: 1 severity + 3 DTC record + 1 status mask.
///
/// # Length
///
/// [`len`](DtcSeverityAndStatusIter::len) counts **complete records**; [`size_hint`](Iterator::size_hint) counts
/// **items yielded**, which is one greater when a partial record trails the buffer (that tail
/// surfaces as a single `Err`, after which the iterator is exhausted). The two therefore differ
/// on malformed input, which is why this deliberately does not implement `ExactSizeIterator` —
/// its `len()` would contradict the inherent one. It does implement
/// [`FusedIterator`](core::iter::FusedIterator).
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

    /// Number of complete records available.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.remaining.len() / 5
    }

    /// Whether there are no complete records.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Collect all triples into a `Vec`.
    ///
    /// # Errors
    /// Returns an error if the byte data contains a partial record.
    #[cfg(feature = "alloc")]
    pub fn collect_all(
        self,
    ) -> Result<alloc::vec::Vec<(DtcSeverityMask, DtcRecord, DtcStatusMask)>, Error> {
        self.collect()
    }
}

impl Iterator for DtcSeverityAndStatusIter<'_> {
    type Item = Result<(DtcSeverityMask, DtcRecord, DtcStatusMask), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        if self.remaining.len() < 5 {
            // See `DtcAndStatusIter::next`: consume the tail so this terminates.
            self.remaining = &[];
            return Some(Err(Error::IncorrectMessageLengthOrInvalidFormat));
        }
        let severity = DtcSeverityMask::from(self.remaining[0]);
        let record = DtcRecord::new(self.remaining[1], self.remaining[2], self.remaining[3]);
        let status = DtcStatusMask::from(self.remaining[4]);
        self.remaining = &self.remaining[5..];
        Some(Ok((severity, record, status)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining.len().div_ceil(5);
        (n, Some(n))
    }
}

impl core::iter::FusedIterator for DtcSeverityAndStatusIter<'_> {}

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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReadDtcInfoResponse<'a> {
    /// Sub-functions 0x01, 0x07: count of DTCs matching a mask.
    NumberOfDtcs {
        /// Sub-function byte echo.
        sub_function_id: u8,
        /// Which status bits this server supports reporting. Same representation as
        /// [`DtcStatusMask`], but a bit is 'on' when the server supports that status — a server
        /// that does not support [`DtcStatusMask::WarningIndicatorRequested`] leaves that bit
        /// 'off' and sets the rest.
        status_availability_mask: DtcStatusMask,
        /// Number of matching DTCs.
        count: u16,
    },
    /// Sub-functions 0x02, 0x0A-0x0E, 0x15: list of `(DtcRecord, DtcStatusMask)` pairs.
    DtcList {
        /// Sub-function byte echo.
        sub_function_id: u8,
        /// Which status bits this server supports reporting. Same representation as
        /// [`DtcStatusMask`], but a bit is 'on' when the server supports that status — a server
        /// that does not support [`DtcStatusMask::WarningIndicatorRequested`] leaves that bit
        /// 'off' and sets the rest.
        status_availability_mask: DtcStatusMask,
        /// Raw record bytes — use [`DtcAndStatusIter`] to iterate.
        #[cfg_attr(feature = "serde", serde(borrow))]
        raw_records: &'a [u8],
    },
    /// Sub-function 0x14: list of DTC fault detection counter records.
    DtcFaultDetectionCounterList {
        /// Raw record bytes — use [`DtcFaultDetectionIter`] to iterate.
        #[cfg_attr(feature = "serde", serde(borrow))]
        raw_records: &'a [u8],
    },
    /// Sub-functions 0x08, 0x09: list of DTC severity records.
    DtcSeverityList {
        /// Sub-function byte echo.
        sub_function_id: u8,
        /// Which status bits this server supports reporting. Same representation as
        /// [`DtcStatusMask`], but a bit is 'on' when the server supports that status — a server
        /// that does not support [`DtcStatusMask::WarningIndicatorRequested`] leaves that bit
        /// 'off' and sets the rest.
        status_availability_mask: DtcStatusMask,
        /// Raw `DTCAndSeverityRecord` bytes (6 bytes each: severity + DTC functional unit +
        /// 3-byte DTC + status). These differ from the 5-byte WWH-OBD records, so
        /// [`DtcSeverityAndStatusIter`] does **not** apply here; no severity-list iterator is
        /// wired yet, so parse these bytes caller-side until one is added.
        #[cfg_attr(feature = "serde", serde(borrow))]
        raw_records: &'a [u8],
    },
    /// Sub-function 0x42: WWH-OBD DTC by mask with severity info.
    WwhObdDtcByMaskRecord {
        /// Functional group identifier echo.
        functional_group_identifier: FunctionalGroupIdentifier,
        /// Which status bits this server supports reporting. Same representation as
        /// [`DtcStatusMask`], but a bit is 'on' when the server supports that status — a server
        /// that does not support [`DtcStatusMask::WarningIndicatorRequested`] leaves that bit
        /// 'off' and sets the rest.
        status_availability_mask: DtcStatusMask,
        /// Severity availability mask.
        severity_availability_mask: DtcSeverityMask,
        /// DTC format identifier.
        format_identifier: DtcFormatIdentifier,
        /// Raw record bytes (5 bytes per record) — use [`DtcSeverityAndStatusIter`].
        #[cfg_attr(feature = "serde", serde(borrow))]
        raw_records: &'a [u8],
    },
}

impl<'a> ReadDtcInfoResponse<'a> {
    /// Iterate `(DtcRecord, DtcStatusMask)` pairs for `DtcList` variants.
    ///
    /// Returns `None` if this is not a `DtcList` variant.
    #[must_use]
    pub fn dtc_and_status_iter(&self) -> Option<DtcAndStatusIter<'a>> {
        match self {
            Self::DtcList { raw_records, .. } => Some(DtcAndStatusIter::new(raw_records)),
            _ => None,
        }
    }

    /// Iterate fault detection counter records for the `DtcFaultDetectionCounterList` variant.
    ///
    /// Returns `None` if this is not that variant.
    #[must_use]
    pub fn fault_detection_iter(&self) -> Option<DtcFaultDetectionIter<'a>> {
        match self {
            Self::DtcFaultDetectionCounterList { raw_records } => {
                Some(DtcFaultDetectionIter::new(raw_records))
            }
            _ => None,
        }
    }

    /// Iterate the severity/DTC/status triples of `reportWWHOBDDTCByMaskRecord` (0x42).
    ///
    /// Returns `None` for every other variant, including
    /// [`DtcSeverityList`](ReadDtcInfoResponse::DtcSeverityList) (0x08/0x09), whose
    /// records are 6 bytes because they carry a `DTCFunctionalUnit` byte this one
    /// does not.
    #[must_use]
    pub fn severity_and_status_iter(&self) -> Option<DtcSeverityAndStatusIter<'a>> {
        match self {
            Self::WwhObdDtcByMaskRecord { raw_records, .. } => {
                Some(DtcSeverityAndStatusIter::new(raw_records))
            }
            _ => None,
        }
    }
}

impl<'a> Decode<'a> for ReadDtcInfoResponse<'a> {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let subfunction_id = buf[0];
        let buf = &buf[1..];

        match subfunction_id {
            0x01 | 0x07 => {
                if buf.len() < 3 {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 4,
                        available: buf.len(),
                    }));
                }
                let status_availability_mask = DtcStatusMask::from(buf[0]);
                let count = u16::from_be_bytes([buf[1], buf[2]]);
                Ok((
                    Self::NumberOfDtcs {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        count,
                    },
                    &buf[3..],
                ))
            }
            0x02 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x15 => {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 2,
                        available: buf.len(),
                    }));
                }
                let status_availability_mask = DtcStatusMask::from(buf[0]);
                Ok((
                    Self::DtcList {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        raw_records: &buf[1..],
                    },
                    &[],
                ))
            }
            0x14 => Ok((Self::DtcFaultDetectionCounterList { raw_records: buf }, &[])),
            0x08 | 0x09 => {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 2,
                        available: buf.len(),
                    }));
                }
                let status_availability_mask = DtcStatusMask::from(buf[0]);
                Ok((
                    Self::DtcSeverityList {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        raw_records: &buf[1..],
                    },
                    &[],
                ))
            }
            0x42 => {
                if buf.len() < 4 {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 5,
                        available: buf.len(),
                    }));
                }
                let functional_group_identifier = FunctionalGroupIdentifier::from(buf[0]);
                let status_availability_mask = DtcStatusMask::from(buf[1]);
                let severity_availability_mask = DtcSeverityMask::from(buf[2]);
                let format_identifier = DtcFormatIdentifier::from(buf[3]);
                Ok((
                    Self::WwhObdDtcByMaskRecord {
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

impl Encode for ReadDtcInfoResponse<'_> {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let mut written = 0;
        match self {
            Self::NumberOfDtcs {
                sub_function_id,
                status_availability_mask,
                count,
            } => {
                written += write_all(writer, &[*sub_function_id, status_availability_mask.bits()])
                    .map_err(Error::io)?;
                written += write_u16_be(writer, *count).map_err(Error::io)?;
            }
            Self::DtcList {
                sub_function_id,
                status_availability_mask,
                raw_records,
            }
            | Self::DtcSeverityList {
                sub_function_id,
                status_availability_mask,
                raw_records,
            } => {
                written += write_all(writer, &[*sub_function_id, status_availability_mask.bits()])
                    .map_err(Error::io)?;
                written += write_all(writer, raw_records).map_err(Error::io)?;
            }
            Self::DtcFaultDetectionCounterList { raw_records } => {
                written += write_u8(writer, 0x14).map_err(Error::io)?;
                written += write_all(writer, raw_records).map_err(Error::io)?;
            }
            Self::WwhObdDtcByMaskRecord {
                functional_group_identifier,
                status_availability_mask,
                severity_availability_mask,
                format_identifier,
                raw_records,
            } => {
                written += write_all(
                    writer,
                    &[
                        0x42,
                        u8::from(*functional_group_identifier),
                        status_availability_mask.bits(),
                        severity_availability_mask.bits(),
                        u8::from(*format_identifier),
                    ],
                )
                .map_err(Error::io)?;
                written += write_all(writer, raw_records).map_err(Error::io)?;
            }
        }
        Ok(written)
    }
}

#[cfg(test)]
mod derive_contract {
    use super::*;
    use crate::test_util::assert_impl_eq;
    #[cfg(feature = "serde")]
    use crate::test_util::assert_impl_serde;

    const _: ReadDtcInfoRequest = ReadDtcInfoRequest::new(
        ReadDtcInfoSubFunction::ReportDtcByStatusMask(DtcStatusMask::TestFailed),
    );

    #[test]
    fn eq_impls() {
        assert_impl_eq::<ReadDtcInfoRequest>();
        assert_impl_eq::<DtcFaultDetectionCounterRecord>();
        assert_impl_eq::<ReadDtcInfoResponse<'static>>();
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_impls() {
        assert_impl_serde::<ReadDtcInfoResponse<'static>>();
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    #[test]
    fn len_counts_complete_records_and_is_empty_agrees() {
        // 4-byte records; a trailing partial (5 bytes = 1 complete + 1 partial byte)
        // counts as one complete record and is surfaced during iteration as an error.
        let one_and_partial = [0x01, 0x02, 0x03, 0x0A, 0xFF];
        let iter = DtcAndStatusIter::new(&one_and_partial);
        assert_eq!(iter.len(), 1);
        assert!(!iter.is_empty());

        // A buffer shorter than one record has zero complete records, and is_empty()
        // agrees with len() == 0 (the previous bug reported is_empty() == false here).
        let partial_only = [0x01, 0x02, 0x03];
        let iter = DtcAndStatusIter::new(&partial_only);
        assert_eq!(iter.len(), 0);
        assert!(iter.is_empty());
    }

    #[test]
    fn all_three_iterators_expose_consistent_len() {
        // 4-byte fault-detection records.
        assert_eq!(DtcFaultDetectionIter::new(&[0u8; 8]).len(), 2);
        assert!(DtcFaultDetectionIter::new(&[0u8; 3]).is_empty());
        // 5-byte severity/DTC/status records.
        assert_eq!(DtcSeverityAndStatusIter::new(&[0u8; 10]).len(), 2);
        assert!(DtcSeverityAndStatusIter::new(&[0u8; 4]).is_empty());
    }

    #[test]
    fn partial_tail_yields_one_error_then_terminates() {
        // Previously `next()` returned `Some(Err(..))` on a partial record *without*
        // advancing `remaining`, so the iterator yielded that error forever: `for _ in iter`
        // looped, `count()` hung, and `collect::<Vec<Result<..>>>()` allocated without bound.
        // `collect_all()` happened to terminate only because `collect::<Result<Vec, _>>()`
        // short-circuits on the first error. Bounded with `take` so a regression fails
        // instead of hanging the suite.
        let data = [0x01, 0x02, 0x03, 0x0A, 0xFF]; // one complete record + 1 stray byte
        let items: heapless_vec::Bounded<8> =
            DtcAndStatusIter::new(&data).take(8).collect_bounded();
        assert_eq!(items.len(), 2, "expected 1 record + 1 error, got {items:?}");
        assert!(items.oks == 1 && items.errs == 1);

        // Same shape for the other two.
        let five: heapless_vec::Bounded<8> = DtcFaultDetectionIter::new(&[0u8; 5])
            .take(8)
            .collect_bounded();
        assert_eq!((five.oks, five.errs), (1, 1));
        let six: heapless_vec::Bounded<8> = DtcSeverityAndStatusIter::new(&[0u8; 6])
            .take(8)
            .collect_bounded();
        assert_eq!((six.oks, six.errs), (1, 1));
    }

    #[test]
    fn iterators_terminate_when_shorter_than_one_record() {
        let mut iter = DtcAndStatusIter::new(&[0x01, 0x02]);
        assert!(matches!(iter.next(), Some(Err(_))));
        assert!(iter.next().is_none(), "iterator must be exhausted");
        assert!(iter.next().is_none(), "and stay exhausted (fused)");
    }

    #[test]
    fn size_hint_matches_the_number_of_items_yielded() {
        for len in 0usize..=12 {
            let data = [0u8; 12];
            let iter = DtcAndStatusIter::new(&data[..len]);
            let (lower, upper) = iter.size_hint();
            let actual = iter.clone().take(16).count();
            assert_eq!(lower, actual, "lower bound wrong for {len} bytes");
            assert_eq!(upper, Some(actual), "upper bound wrong for {len} bytes");
        }
    }

    /// Minimal counting collector so the termination tests need no allocator.
    mod heapless_vec {
        use super::*;
        use core::fmt;

        pub struct Bounded<const N: usize> {
            pub oks: usize,
            pub errs: usize,
        }

        impl<const N: usize> Bounded<N> {
            pub fn len(&self) -> usize {
                self.oks + self.errs
            }
        }

        impl<const N: usize> fmt::Debug for Bounded<N> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} ok, {} err", self.oks, self.errs)
            }
        }

        pub trait CollectBounded {
            fn collect_bounded<const N: usize>(self) -> Bounded<N>;
        }

        impl<I, T> CollectBounded for I
        where
            I: Iterator<Item = Result<T, Error>>,
        {
            fn collect_bounded<const N: usize>(self) -> Bounded<N> {
                let mut b = Bounded::<N> { oks: 0, errs: 0 };
                for item in self {
                    match item {
                        Ok(_) => b.oks += 1,
                        Err(_) => b.errs += 1,
                    }
                }
                b
            }
        }
    }
    use heapless_vec::CollectBounded;
}
