//! `ReadDTCInformation` (0x19) request and response service implementation

use automotive_wire_codec::{read_u8, write_all, write_u8, write_u16_be};

use crate::shared::{fuse_sprmib, split_sprmib};
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
    /// Whether the server should suppress a positive response (SPRMIB).
    ///
    /// ISO 14229-1:2020 Table 13 requires a server to support both values for every
    /// sub-function it supports, so this is independent of `dtc_subfunction`.
    pub suppress_positive_response: bool,
    /// The sub-function specifying what DTC information to report.
    pub dtc_subfunction: ReadDtcInfoSubFunction,
}

impl ReadDtcInfoRequest {
    /// Create a new `ReadDtcInfoRequest`.
    #[must_use]
    pub const fn new(
        suppress_positive_response: bool,
        dtc_subfunction: ReadDtcInfoSubFunction,
    ) -> Self {
        Self {
            suppress_positive_response,
            dtc_subfunction,
        }
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
        // The sub-function byte carries SPRMIB in bit 7, so it is written here rather than by
        // `ReadDtcInfoSubFunction::encode`, which has no way to know the flag.
        let sub_function = fuse_sprmib(
            self.suppress_positive_response,
            self.dtc_subfunction.value(),
        );
        let mut written = write_u8(writer, sub_function).map_err(Error::io)?;
        written += self.dtc_subfunction.encode_parameters(writer)?;
        Ok(written)
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
        // Bit 7 is SPRMIB, not part of the sub-function value (ISO 14229-1:2020 Table 13).
        let (suppress_positive_response, sub) = split_sprmib(buf[0]);
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
                let (mem, r) = read_u8(r)?;
                (S::ReportUserDefMemoryDtcByStatusMask(m, mem), r)
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
        Ok((
            ReadDtcInfoRequest::new(suppress_positive_response, dtc_subfunction),
            rest,
        ))
    }
}

#[cfg(test)]
mod read_dtc_info_request_encode_tests {
    use super::*;
    use crate::{NegativeResponseCode, test_util::assert_encode_size_agrees};

    #[test]
    fn encode_no_param_subfunction() {
        // 0x0A ReportSupportedDtc, no parameters.
        let req = ReadDtcInfoRequest::new(false, ReadDtcInfoSubFunction::ReportSupportedDtc);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x0A]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_single_param_subfunction() {
        // 0x02 ReportDtcByStatusMask(mask). DtcStatusMask is 1 byte.
        let mask = DtcStatusMask::from(0xFF);
        let req =
            ReadDtcInfoRequest::new(false, ReadDtcInfoSubFunction::ReportDtcByStatusMask(mask));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn both_sprmib_values_round_trip_through_the_request_frame() {
        // ISO 14229-1:2020 Table 13 requires that "values of both '0' and '1' shall be
        // supported for all SubFunction parameter values ... supported by the server for any
        // given service", and clause 12.3.2.2 introduces 0x19's sub-function table with
        // "(suppressPosRspMsgIndicationBit (bit 7) not shown)". A suppressed
        // reportDTCByStatusMask used to be rejected with TrailingBytes, because the raw 0x82
        // fell through to IsoSaeReserved, which consumes no payload.
        for (wire, suppressed) in [
            ([0x19, 0x02, 0xFF].as_slice(), false),
            ([0x19, 0x82, 0xFF].as_slice(), true),
        ] {
            let (req, _) = crate::Request::decode(wire).unwrap();
            assert_eq!(
                req.is_positive_response_suppressed(),
                suppressed,
                "wrong SPRMIB for {wire:02X?}"
            );
            let mut buf = [0u8; 8];
            let written = req.encode_to_slice(&mut buf).unwrap();
            assert_eq!(&buf[..written], wire, "round trip failed for {wire:02X?}");
        }
    }

    #[test]
    fn a_suppressed_sub_function_is_not_mistaken_for_a_reserved_one() {
        // 0x8A is reportSupportedDTC with SPRMIB set. Matching on the raw byte decoded this as
        // IsoSaeReserved(0x8A) and reported suppression as false, so a server built on this
        // answered SubFunctionNotSupported to a request it was required to execute.
        let (req, _) = <ReadDtcInfoRequest as Decode>::decode(&[0x8A]).unwrap();
        assert_eq!(
            req.dtc_subfunction,
            ReadDtcInfoSubFunction::ReportSupportedDtc
        );
        assert!(req.suppress_positive_response);
    }

    #[test]
    fn user_def_memory_dtc_by_status_mask_carries_a_memory_selection() {
        // ISO 14229-1:2020 Table 310 marks both DTCStatusMask and MemorySelection `M`. The
        // sibling sub-functions 0x18 and 0x19 (Tables 311/312) already carried theirs.
        let wire = [0x17, 0xFF, 0x01];
        let (req, rest) = <ReadDtcInfoRequest as Decode>::decode(&wire).unwrap();
        assert_eq!(
            req.dtc_subfunction,
            ReadDtcInfoSubFunction::ReportUserDefMemoryDtcByStatusMask(
                DtcStatusMask::from(0xFF),
                0x01
            )
        );
        assert!(rest.is_empty());

        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn user_def_memory_dtc_by_status_mask_rejects_a_missing_memory_selection() {
        // Without the MemorySelection byte the frame is malformed, and used to be accepted
        // while the conformant 3-parameter form was rejected as having trailing bytes.
        assert!(<ReadDtcInfoRequest as Decode>::decode_exact(&[0x17, 0xFF]).is_err());
    }

    #[test]
    fn encode_multi_param_subfunction() {
        // 0x42 ReportWwhObdDtcByMaskRecord(group, status, severity).
        let req = ReadDtcInfoRequest::new(
            false,
            ReadDtcInfoSubFunction::ReportWwhObdDtcByMaskRecord(
                FunctionalGroupIdentifier::EmissionsSystemGroup,
                DtcStatusMask::from(0x08),
                DtcSeverityMask::CheckImmediately,
            ),
        );
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x42, 0x33, 0x08, 0b1000_0000]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_reserved_subfunction() {
        // IsoSaeReserved carries the sub-function byte itself, no params.
        let req = ReadDtcInfoRequest::new(false, ReadDtcInfoSubFunction::IsoSaeReserved(0x57));
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
            ReadDtcInfoRequest::new(false, ReadDtcInfoSubFunction::ReportSupportedDtc),
            ReadDtcInfoRequest::new(
                false,
                ReadDtcInfoSubFunction::ReportDtcByStatusMask(DtcStatusMask::from(0xFF)),
            ),
            ReadDtcInfoRequest::new(
                false,
                ReadDtcInfoSubFunction::ReportWwhObdDtcByMaskRecord(
                    FunctionalGroupIdentifier::EmissionsSystemGroup,
                    DtcStatusMask::from(0x08),
                    DtcSeverityMask::CheckImmediately,
                ),
            ),
            ReadDtcInfoRequest::new(false, ReadDtcInfoSubFunction::IsoSaeReserved(0x57)),
            ReadDtcInfoRequest::new(
                false,
                ReadDtcInfoSubFunction::ReportDtcSnapshotRecordByDtcNumber(
                    DtcRecord::new(0x12, 0x34, 0x56),
                    DtcSnapshotRecordNumber::new(0x01),
                ),
            ),
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
#[non_exhaustive]
pub struct DtcFaultDetectionCounterRecord {
    /// The DTC this counter belongs to.
    pub dtc_record: DtcRecord,
    /// Fault detection counter, used for non-emissions related servers.
    pub dtc_fault_detection_counter: u8,
}

impl DtcFaultDetectionCounterRecord {
    /// Create a `DtcFaultDetectionCounterRecord`.
    ///
    /// This type is `#[non_exhaustive]`, so downstream crates cannot use a struct literal and
    /// need this constructor — for a test fixture, or for a server building the record list
    /// that [`DtcFaultDetectionIter`] reads back.
    #[must_use]
    pub const fn new(dtc_record: DtcRecord, dtc_fault_detection_counter: u8) -> Self {
        Self {
            dtc_record,
            dtc_fault_detection_counter,
        }
    }
}

/// Subfunctions for the `ReadDTCInformation` service
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
    /// * Parameter: `memorySelection`(1) — addresses the user-defined DTC memory to read from
    ///
    /// Both parameters are mandatory (ISO 14229-1:2020 Table 310).
    ///
    /// 0x17
    ReportUserDefMemoryDtcByStatusMask(DtcStatusMask, u8),

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
    /// A sub-function byte this crate does not model.
    ///
    /// ISO 14229-1:2020 Table 317 reserves `0x00`, `0x1B`-`0x41`, `0x43`-`0x54` and
    /// `0x57`-`0x7F`. The remaining bytes that land here are report types the crate has not
    /// implemented yet; a server should answer those with
    /// [`NegativeResponseCode::SubFunctionNotSupported`].
    ///
    /// The value never has bit 7 set: that bit is SPRMIB and is split off into
    /// [`ReadDtcInfoRequest::suppress_positive_response`] before the sub-function is decoded.
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
            Self::ReportUserDefMemoryDtcByStatusMask(_, _) => 0x17,
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

impl ReadDtcInfoSubFunction {
    /// Write only this sub-function's parameter bytes, not its leading sub-function byte.
    ///
    /// [`ReadDtcInfoRequest::encode`] writes that byte itself, because it has to fuse SPRMIB
    /// into bit 7 and this type does not carry the flag.
    fn encode_parameters(self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        use ReadDtcInfoSubFunction as S;
        let mut written = 0;
        match self {
            S::ReportNumberOfDtcByStatusMask(m) | S::ReportDtcByStatusMask(m) => {
                written += m.encode(writer)?;
            }
            S::ReportUserDefMemoryDtcByStatusMask(m, mem) => {
                written += m.encode(writer)?;
                written += write_u8(writer, mem).map_err(Error::io)?;
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
                written += write_u8(writer, mem).map_err(Error::io)?;
            }
            S::ReportUserDefMemoryDtcExtDataRecordByDtcNumber(r, n, mem) => {
                written += r.encode(writer)?;
                written += n.encode(writer)?;
                written += write_u8(writer, mem).map_err(Error::io)?;
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
                written += write_u8(writer, rg).map_err(Error::io)?;
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

impl Encode for ReadDtcInfoSubFunction {
    type Error = crate::Error;

    /// Writes the sub-function byte with SPRMIB clear, followed by this sub-function's
    /// parameters.
    ///
    /// Encode a [`ReadDtcInfoRequest`] instead to control the suppress-positive-response bit;
    /// this impl always leaves it clear, because the flag lives on the request rather than on
    /// the sub-function.
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let mut written = write_u8(writer, self.value()).map_err(Error::io)?;
        written += self.encode_parameters(writer)?;
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

/// Lazy iterator over the WWH-OBD `(DtcSeverityMask, DtcRecord, DtcStatusMask)` triples of a
/// [`ReadDtcInfoResponse::WwhObdDtcByMaskRecord`] (sub-function `0x42`).
///
/// Each triple is 5 bytes: 1 severity + 3 DTC record + 1 status mask.
///
/// This applies **only** to the WWH-OBD variant. The `0x08`/`0x09`
/// [`DtcSeverityList`](ReadDtcInfoResponse::DtcSeverityList) records are 6 bytes and carry an
/// extra DTC functional-unit byte, so they need a different iterator (not yet wired).
///
/// # Length
///
/// [`len`](WwhObdDtcSeverityIter::len) counts **complete records**; [`size_hint`](Iterator::size_hint) counts
/// **items yielded**, which is one greater when a partial record trails the buffer (that tail
/// surfaces as a single `Err`, after which the iterator is exhausted). The two therefore differ
/// on malformed input, which is why this deliberately does not implement `ExactSizeIterator` —
/// its `len()` would contradict the inherent one. It does implement
/// [`FusedIterator`](core::iter::FusedIterator).
#[derive(Clone, Debug)]
pub struct WwhObdDtcSeverityIter<'a> {
    remaining: &'a [u8],
}

impl<'a> WwhObdDtcSeverityIter<'a> {
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

impl Iterator for WwhObdDtcSeverityIter<'_> {
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

impl core::iter::FusedIterator for WwhObdDtcSeverityIter<'_> {}

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
        /// How the server's DTC numbers are formatted and encoded.
        ///
        /// Mandatory in this response (ISO 14229-1:2020 Table 319), and the only thing that
        /// says how to interpret the three bytes of each DTC — ISO 14229-1 itself defines no
        /// decoding method for them.
        format_identifier: DtcFormatIdentifier,
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
        /// Raw record bytes, 4 per record (3-byte DTC + status) — use [`DtcAndStatusIter`].
        /// Decoding rejects a length that is not a whole number of records.
        #[cfg_attr(feature = "serde", serde(borrow))]
        raw_records: &'a [u8],
    },
    /// Sub-function 0x14: list of DTC fault detection counter records.
    DtcFaultDetectionCounterList {
        /// Raw record bytes, 4 per record (3-byte DTC + counter) — use
        /// [`DtcFaultDetectionIter`]. Decoding rejects a length that is not a whole number of
        /// records.
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
        /// Raw `DTCAndSeverityRecord` bytes: 6 each — severity + DTC functional unit +
        /// 3-byte DTC + status. No iterator is wired for this variant yet, so parse them
        /// caller-side. Note these are *not* the 5-byte WWH-OBD records read by
        /// [`WwhObdDtcSeverityIter`]. Decoding rejects a length that is not a multiple of 6.
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
        /// Raw record bytes, 5 per record — use [`WwhObdDtcSeverityIter`]. Decoding rejects a length
        /// that is not a whole number of records.
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
    pub fn dtc_fault_detection_iter(&self) -> Option<DtcFaultDetectionIter<'a>> {
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
    pub fn wwh_obd_dtc_severity_iter(&self) -> Option<WwhObdDtcSeverityIter<'a>> {
        match self {
            Self::WwhObdDtcByMaskRecord { raw_records, .. } => {
                Some(WwhObdDtcSeverityIter::new(raw_records))
            }
            _ => None,
        }
    }
}

/// Validate that a record list divides evenly into whole records.
///
/// A UDS frame is complete when it arrives — its length comes from the transport — so a
/// trailing partial record means the frame is malformed, not that more bytes are coming.
/// Rejecting it here matches how the crate treats every other length mismatch.
///
/// Consequently the iterators reached from a **decoded** [`ReadDtcInfoResponse`] never see a
/// partial tail. A hand-constructed variant still can — the enum's `#[non_exhaustive]` stops
/// exhaustive matching, not variant construction, and `Encode` writes `raw_records` verbatim —
/// so the iterators keep their `Result` item type and their one-error-then-terminate behaviour.
fn whole_records(raw: &[u8], record_len: usize) -> Result<&[u8], Error> {
    if raw.len() % record_len == 0 {
        Ok(raw)
    } else {
        Err(Error::IncorrectMessageLengthOrInvalidFormat)
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
                // Table 319: DTCStatusAvailabilityMask, DTCFormatIdentifier, then a 2-byte
                // DTCCount -- four mandatory bytes after the sub-function echo.
                if buf.len() < 4 {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 4,
                        available: buf.len(),
                    }));
                }
                let status_availability_mask = DtcStatusMask::from(buf[0]);
                let format_identifier = DtcFormatIdentifier::from(buf[1]);
                let count = u16::from_be_bytes([buf[2], buf[3]]);
                Ok((
                    Self::NumberOfDtcs {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        format_identifier,
                        count,
                    },
                    &buf[4..],
                ))
            }
            0x02 | 0x0A | 0x0B | 0x0C | 0x0D | 0x0E | 0x15 => {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 1,
                        available: buf.len(),
                    }));
                }
                let status_availability_mask = DtcStatusMask::from(buf[0]);
                Ok((
                    Self::DtcList {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        raw_records: whole_records(&buf[1..], 4)?,
                    },
                    &[],
                ))
            }
            0x14 => Ok((
                Self::DtcFaultDetectionCounterList {
                    raw_records: whole_records(buf, 4)?,
                },
                &[],
            )),
            0x08 | 0x09 => {
                if buf.is_empty() {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 1,
                        available: buf.len(),
                    }));
                }
                let status_availability_mask = DtcStatusMask::from(buf[0]);
                Ok((
                    Self::DtcSeverityList {
                        sub_function_id: subfunction_id,
                        status_availability_mask,
                        raw_records: whole_records(&buf[1..], 6)?,
                    },
                    &[],
                ))
            }
            0x42 => {
                if buf.len() < 4 {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 4,
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
                        raw_records: whole_records(&buf[4..], 5)?,
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
                format_identifier,
                count,
            } => {
                written += write_all(
                    writer,
                    &[
                        *sub_function_id,
                        status_availability_mask.bits(),
                        u8::from(*format_identifier),
                    ],
                )
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
        false,
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
mod response_decode_tests {
    use super::*;
    use crate::{Decode, Encode, Response};

    #[test]
    fn the_dtc_count_response_carries_a_format_identifier() {
        // ISO 14229-1:2020 Table 341 (flow example #1) is exactly these six bytes:
        // SID, reportType, DTCStatusAvailabilityMask, DTCFormatIdentifier, count high, low.
        // Table 319 marks all of them `M`. The format identifier used to be missing from the
        // model, so it was read as the count high byte: this frame was rejected outright, and
        // a count of 1 came back as 0x0100.
        let wire = [0x59, 0x01, 0x2F, 0x01, 0x00, 0x01];
        let (resp, _) = Response::decode(&wire).unwrap();
        let Response::ReadDtcInfo(ReadDtcInfoResponse::NumberOfDtcs {
            sub_function_id,
            status_availability_mask,
            format_identifier,
            count,
        }) = resp
        else {
            panic!("expected a NumberOfDtcs response, got {resp:?}");
        };
        assert_eq!(sub_function_id, 0x01);
        assert_eq!(status_availability_mask.bits(), 0x2F);
        assert_eq!(
            format_identifier,
            DtcFormatIdentifier::Iso14229_1DtcFormat,
            "0x01 is ISO_14229-1_DTCFormat per Table D.14"
        );
        assert_eq!(count, 1);

        let mut buf = [0u8; 8];
        let written = resp.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn the_wwh_obd_response_header_and_records_decode_field_by_field() {
        // ISO 14229-1:2020 Table 332: FGID, DTCStatusAvailabilityMask,
        // DTCSeverityAvailabilityMask, DTCFormatIdentifier, then 5-byte records of
        // severity + 3-byte DTC + status.
        //
        // Nothing had ever asserted a value this iterator yields, nor any of the four header
        // fields — every test checked only counts and `is_ok()`. Reversing the header fields, or
        // swapping severity with status and reversing the DTC bytes, passed the whole suite.
        let wire = [
            0x59, 0x42, 0x33, 0xFF, 0xF0, 0x01, 0x80, 0x01, 0x02, 0x03, 0x0A,
        ];
        let (resp, _) = Response::decode(&wire).unwrap();
        let Response::ReadDtcInfo(ReadDtcInfoResponse::WwhObdDtcByMaskRecord {
            functional_group_identifier,
            status_availability_mask,
            severity_availability_mask,
            format_identifier,
            ..
        }) = resp
        else {
            panic!("expected a WwhObdDtcByMaskRecord response, got {resp:?}");
        };

        assert_eq!(
            functional_group_identifier,
            FunctionalGroupIdentifier::EmissionsSystemGroup,
            "0x33 is the emissions group per Table D.15"
        );
        assert_eq!(status_availability_mask.bits(), 0xFF);
        assert_eq!(severity_availability_mask.bits(), 0xF0);
        assert_eq!(format_identifier, DtcFormatIdentifier::Iso14229_1DtcFormat);

        let Response::ReadDtcInfo(ref inner) = resp else {
            unreachable!()
        };
        let mut records = inner
            .wwh_obd_dtc_severity_iter()
            .expect("0x42 carries this iterator");
        let (severity, dtc, status) = records.next().unwrap().unwrap();
        assert_eq!(
            severity,
            DtcSeverityMask::CheckImmediately,
            "severity is byte 0"
        );
        assert_eq!(
            dtc,
            DtcRecord::new(0x01, 0x02, 0x03),
            "DTC bytes are big-endian"
        );
        assert_eq!(
            status.bits(),
            0x0A,
            "status is the last byte, not the first"
        );
        assert!(records.next().is_none());

        let mut buf = [0u8; 16];
        let written = resp.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn an_insufficient_data_shortfall_measures_both_counts_on_one_buffer() {
        // `needed` used to include the sub-function byte while `available` was measured after it
        // was sliced off, so a caller computing `needed - available` got a shortfall one byte
        // too large. Both are now relative to the payload.
        for (label, frame, needed, available) in [
            ("0x01 count", [0x59, 0x01, 0x2F].as_slice(), 4, 1),
            ("0x02 list", [0x59, 0x02].as_slice(), 1, 0),
            ("0x08 severity list", [0x59, 0x08].as_slice(), 1, 0),
            ("0x42 WWH-OBD", [0x59, 0x42, 0x33, 0xFF].as_slice(), 4, 2),
        ] {
            let got = Response::decode(frame);
            let Err(Error::InsufficientData(incomplete)) = got else {
                panic!("{label}: expected InsufficientData, got {got:?}");
            };
            assert_eq!(incomplete.needed, needed, "{label}: wrong `needed`");
            assert_eq!(
                incomplete.available, available,
                "{label}: wrong `available`"
            );
        }
    }

    #[test]
    fn a_dtc_count_response_missing_the_count_is_rejected() {
        // Four payload bytes are mandatory after the sub-function echo; three is a truncated
        // frame, not a frame whose format identifier happens to be the count's high byte.
        assert!(<ReadDtcInfoResponse as Decode>::decode(&[0x01, 0x2F, 0x01]).is_err());
    }

    /// `(label, sub-function payload prefix, record width)` for every variant that carries a
    /// record list. The payload here is what follows the sub-function byte.
    const LISTS: [(&str, &[u8], usize); 4] = [
        // 0x02: status availability mask, then 4-byte (DTC, status) records.
        ("DtcList 0x02", &[0x02, 0xFF], 4),
        // 0x14: no mask byte — records start immediately.
        ("DtcFaultDetectionCounterList 0x14", &[0x14], 4),
        // 0x08: status availability mask, then 6-byte DTCAndSeverityRecord entries.
        ("DtcSeverityList 0x08", &[0x08, 0xFF], 6),
        // 0x42: fgid + status mask + severity mask + format id, then 5-byte WWH-OBD records.
        (
            "WwhObdDtcByMaskRecord 0x42",
            &[0x42, 0x33, 0xFF, 0xF0, 0x01],
            5,
        ),
    ];

    /// Enough for the widest prefix (5 bytes) plus the longest record list these tests build
    /// (3 x 6-byte records).
    const FRAME_CAP: usize = 32;

    /// Build a frame into a fixed-size buffer, returning it with its used length. A stack buffer
    /// rather than a `Vec` so the tests compile without the `alloc` feature.
    fn frame(prefix: &[u8], record_bytes: usize) -> ([u8; FRAME_CAP], usize) {
        let len = prefix.len() + record_bytes;
        assert!(len <= FRAME_CAP, "frame does not fit the test buffer");
        let mut buf = [0u8; FRAME_CAP];
        buf[..prefix.len()].copy_from_slice(prefix);
        for (i, byte) in buf[prefix.len()..len].iter_mut().enumerate() {
            *byte = u8::try_from(i % 251).unwrap_or(0);
        }
        (buf, len)
    }

    #[test]
    fn record_lists_must_divide_evenly_into_records() {
        // A trailing partial record means the frame is malformed. Rejecting it here matches how
        // the crate treats every other length mismatch, and means the iterators returned by the
        // accessors can never see a partial tail.
        // Every misaligned length below two whole records, which crucially includes 1..width --
        // a list too short to hold even one record. That range was untested, so a `whole_records`
        // that accepted any short list passed: 0 records was covered by
        // `empty_record_lists_are_valid` and 1 record by the aligned test, but the 0-valid /
        // 1-invalid boundary, the whole point of the check, was not.
        for (label, prefix, width) in LISTS {
            for record_bytes in (1..2 * width).filter(|n| n % width != 0) {
                let (buf, len) = frame(prefix, record_bytes);
                let got = <ReadDtcInfoResponse as Decode>::decode(&buf[..len]);
                assert!(
                    matches!(got, Err(Error::IncorrectMessageLengthOrInvalidFormat)),
                    "{label}: {record_bytes} record bytes (width {width}) should be rejected, got {got:?}"
                );
            }
        }
    }

    #[test]
    fn aligned_record_lists_decode_with_the_expected_record_count() {
        for (label, prefix, width) in LISTS {
            for records in 0..=3usize {
                let (buf, len) = frame(prefix, width * records);
                let (resp, _) = <ReadDtcInfoResponse as Decode>::decode(&buf[..len])
                    .unwrap_or_else(|e| panic!("{label}: {records} records should decode: {e:?}"));
                let counted = resp
                    .dtc_and_status_iter()
                    .map(Iterator::count)
                    .or_else(|| resp.dtc_fault_detection_iter().map(Iterator::count))
                    .or_else(|| resp.wwh_obd_dtc_severity_iter().map(Iterator::count));
                // DtcSeverityList has no iterator wired yet, so it has no count to check.
                if let Some(counted) = counted {
                    assert_eq!(counted, records, "{label}: wrong record count");
                }
            }
        }
    }

    #[test]
    fn empty_record_lists_are_valid() {
        // A server with no matching DTCs answers with the header and no records.
        for (label, prefix, _) in LISTS {
            let got = <ReadDtcInfoResponse as Decode>::decode(prefix);
            assert!(
                got.is_ok(),
                "{label}: empty record list must decode, got {got:?}"
            );
        }
    }

    #[test]
    fn a_misaligned_list_is_rejected_at_the_frame_layer_too() {
        // SID 0x59, sub 0x02, mask 0xFF, then 5 record bytes — one byte past a whole record.
        let wire = [0x59, 0x02, 0xFF, 0x01, 0x02, 0x03, 0x0A, 0xEE];
        assert!(matches!(
            Response::decode(&wire),
            Err(Error::IncorrectMessageLengthOrInvalidFormat)
        ));
        // The aligned frame still round-trips.
        let wire = [0x59, 0x02, 0xFF, 0x01, 0x02, 0x03, 0x0A];
        let (resp, _) = Response::decode(&wire).unwrap();
        let mut buf = [0u8; 16];
        let n = resp.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..n], &wire);
    }

    #[test]
    fn iterators_from_a_decoded_response_never_yield_an_error() {
        // The payoff: with decode validating alignment, every iterator obtained through an
        // accessor is error-free. The `Err` arm remains reachable only via `Iter::new` on
        // arbitrary bytes, which `iter_tests` covers.
        for (label, prefix, width) in LISTS {
            let (buf, len) = frame(prefix, width * 2);
            let (resp, _) = <ReadDtcInfoResponse as Decode>::decode(&buf[..len]).unwrap();
            if let Some(mut it) = resp.dtc_and_status_iter() {
                assert!(it.all(|r| r.is_ok()), "{label}");
            }
            if let Some(mut it) = resp.dtc_fault_detection_iter() {
                assert!(it.all(|r| r.is_ok()), "{label}");
            }
            if let Some(mut it) = resp.wwh_obd_dtc_severity_iter() {
                assert!(it.all(|r| r.is_ok()), "{label}");
            }
        }
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
        assert_eq!(WwhObdDtcSeverityIter::new(&[0u8; 10]).len(), 2);
        assert!(WwhObdDtcSeverityIter::new(&[0u8; 4]).is_empty());
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
        let six: heapless_vec::Bounded<8> = WwhObdDtcSeverityIter::new(&[0u8; 6])
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
        // All three iterators, every buffer length across several record boundaries. The two
        // record widths differ (4 bytes vs 5), so each needs its own `div_ceil` checked.
        // `take` bounds the count so a non-termination regression fails rather than hangs.
        let data = [0u8; 16];
        for len in 0usize..=16 {
            let it = DtcAndStatusIter::new(&data[..len]);
            let actual = it.clone().take(32).count();
            assert_eq!(
                it.size_hint(),
                (actual, Some(actual)),
                "DtcAndStatusIter, {len} bytes"
            );
            assert_eq!(
                actual,
                len.div_ceil(4),
                "DtcAndStatusIter count, {len} bytes"
            );

            let it = DtcFaultDetectionIter::new(&data[..len]);
            let actual = it.clone().take(32).count();
            assert_eq!(
                it.size_hint(),
                (actual, Some(actual)),
                "DtcFaultDetectionIter, {len} bytes"
            );
            assert_eq!(
                actual,
                len.div_ceil(4),
                "DtcFaultDetectionIter count, {len} bytes"
            );

            let it = WwhObdDtcSeverityIter::new(&data[..len]);
            let actual = it.clone().take(32).count();
            assert_eq!(
                it.size_hint(),
                (actual, Some(actual)),
                "WwhObdDtcSeverityIter, {len} bytes"
            );
            assert_eq!(
                actual,
                len.div_ceil(5),
                "WwhObdDtcSeverityIter count, {len} bytes"
            );
        }
    }

    #[test]
    fn all_three_iterators_terminate_and_stay_exhausted() {
        // FusedIterator is only sound if `next()` keeps returning None once drained.
        //
        // The drains are bounded, like the ones in `size_hint_matches_the_number_of_items_
        // yielded`: a 7-byte buffer can yield at most 2 items, so `take(8)` cannot hide a real
        // result, and a regression that stops advancing `remaining` fails here instead of
        // hanging `cargo test` forever with the other failures unreported.
        const CAP: usize = 8;
        let data = [0u8; 7]; // not a whole number of records for either width

        let mut a = DtcAndStatusIter::new(&data);
        assert!(
            a.by_ref().take(CAP).count() < CAP,
            "DtcAndStatusIter did not terminate"
        );
        assert!(a.next().is_none() && a.next().is_none());

        let mut b = DtcFaultDetectionIter::new(&data);
        assert!(
            b.by_ref().take(CAP).count() < CAP,
            "DtcFaultDetectionIter did not terminate"
        );
        assert!(b.next().is_none() && b.next().is_none());

        let mut c = WwhObdDtcSeverityIter::new(&data);
        assert!(
            c.by_ref().take(CAP).count() < CAP,
            "WwhObdDtcSeverityIter did not terminate"
        );
        assert!(c.next().is_none() && c.next().is_none());
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
