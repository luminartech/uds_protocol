use bitmask_enum::bitmask;

use crate::{Decode, DecodeIter, Encode, Error, Incomplete};
use automotive_wire_codec::{write_all, write_u8};

/// Bit-packed DTC status information used by the `ReadDTCInformation` service
///
/// `DtcStatusMask` (1 byte)
/// 8 DTC status bits. Refer to D.2
/// A DTC status matches the mask if any one of the DTCs actual status bits is set to `1`
/// and the corresponding on in the mask is set to 1
/// if( `DtcStatusMask` & `DTCStatus` = !0) is a match
///
/// Server note:
///     If the mask uses bits that the server does not support,
///     the server shall process the bits it does support and ignore the rest
///
/// ```
/// use uds_protocol::{DtcStatusMask, ReadDtcInfoSubFunction};
/// // Get DTCs with TestFailed and PendingDtc statuses
/// let dtc_status = DtcStatusMask::TestFailed | DtcStatusMask::PendingDtc;
/// let dtc_subfunction = ReadDtcInfoSubFunction::ReportNumberOfDtcByStatusMask(dtc_status);
/// ```
///
/// Per DTC statuses
///
/// | DTC Status Bit | DTC Status Name | Bit state after ClearDiagnosticInformation|
/// | - | ------------------------------ | --- |
/// | 0 | [`TestFailed`](DtcStatusMask::TestFailed)                         | **0** |
/// | 1 | [`TestFailedThisOperationCycle`](DtcStatusMask::TestFailedThisOperationCycle)       | **0** |
/// | 2 | [`PendingDtc`](DtcStatusMask::PendingDtc)                         | **0** |
/// | 3 | [`ConfirmedDtc`](DtcStatusMask::ConfirmedDtc)                       | **0** |
/// | 4 | [`TestNotCompletedSinceLastClear`](DtcStatusMask::TestNotCompletedSinceLastClear)     | **1** |
/// | 5 | [`TestFailedSinceLastClear`](DtcStatusMask::TestFailedSinceLastClear)           | **0** |
/// | 6 | [`TestNotCompletedThisOperationCycle`](DtcStatusMask::TestNotCompletedThisOperationCycle) | **1** |
/// | 7 | [`WarningIndicatorRequested`](DtcStatusMask::WarningIndicatorRequested)          | **0** |
#[bitmask(u8)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum DtcStatusMask {
    /// Status of the most recently performed test.
    ///
    /// Bit state definition:
    /// * 0 shall indicate the last test passed
    /// * 1 shall indicate the last matured test **failed**
    ///
    /// Will be 0 after a successful [`ClearDiagnosticInfoRequest`](crate::ClearDiagnosticInfoRequest) service
    TestFailed,
    /// Whether or not a diagnostic test has reported a test failed result during the current operation cycle,
    /// or that it's been reported during this operation and after `ClearDiagnosticInformation`
    ///
    /// Bit state definition:
    /// * 0 shall indicate that **no test failed** during the current operation cycle or after a `ClearDiagnosticInformation`
    /// * 1 shall indicate that a test failed during the current operation cycle or after a `ClearDiagnosticInformation`
    ///
    /// Shall remain a 1 until a new operation cycle is started
    TestFailedThisOperationCycle,

    /// Similar to [`Self::TestFailedThisOperationCycle`], but will only clear after
    /// a cycle is finished and there is a passed test w/ no failure
    ///
    /// Bit state definition:
    /// * 0 -  Test passed **with no failure** after completing a cycle
    /// * 1 -  Test failed during the current operation cycle
    PendingDtc,

    /// Indicates whether a malfunction was detected enough times to warrant the DTC being stored
    /// in long term memory. This doesn't mean that the DTC failure is present at the time of the request.
    /// Aging threshold for clearing itself depends on the vehicle manufacturer or OBD regulations
    ///
    /// Bit state definition:
    /// * 0 - DTC has **never been confirmed** since last `ClearDiagnosticInformation`, or after aging criteria have been met
    /// * 1 - DTC has been confirmed at least once
    ConfirmedDtc,

    /// Indicates whether a test has run and completed since last `ClearDiagnosticInformation`
    /// Will not reset to 1 by any method other than calling `ClearDiagnosticInformation`
    ///
    /// Bit state definition:
    /// * 0 - Test has returned passed or failed at least once since last `ClearDiagnosticInformation`
    /// * 1 - Test has **not** run to completion
    TestNotCompletedSinceLastClear,

    /// Indicates whether a test has failed since the last `ClearDiagnosticInformation`
    /// This is a latched [`Self::TestFailedThisOperationCycle`]
    /// Vehicle manufacturer is in charge of clearing this bit if there is an aging threshold is fulfilled
    ///
    /// Bit state definition:
    /// * 0 - Test has **not** failed since last `ClearDiagnosticInformation`
    /// * 1 - Test has failed at least once since last `ClearDiagnosticInformation`
    TestFailedSinceLastClear,

    /// Indicates whether a test has run and completed during the current operation cycle,
    ///     or whether is has run and completed after the last `ClearDiagnosticInformation` during the current operation cycle
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

impl Encode for DtcStatusMask {
    type Error = crate::Error;
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, self.bits()).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for DtcStatusMask {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}

/// Specifies the format of the DTC reported by the server.
///
/// A given server shall only support one `DtcFormatIdentifier`.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum DtcFormatIdentifier {
    /// Defined in [SAE J2012-DA](<https://www.sae.org/standards/content/j2012da_202403/>) DTC Format
    SaeJ2012DaDtcFormat00 = 0x00,

    /// reported for `DTCAndStatusRecord`
    Iso14229_1DtcFormat = 0x01,

    /// Defined in [SAE J1939-73](<https://www.sae.org/standards/content/j1939/73_202208/>)
    SaeJ1939_73DtcFormat = 0x02,

    /// Defined in [ISO-11992](<https://www.iso.org/standard/33992.html>)
    Iso11992_4DtcFormat = 0x03,

    /// Defined in SAE J2012-DA](<https://www.sae.org/standards/content/j2012da_202403/>)
    SaeJ2012DaDtcFormat04 = 0x04,

    /// Reserved for future usage
    /// 0x05 - 0xFF
    IsoSaeReserved(u8),
}

impl From<u8> for DtcFormatIdentifier {
    fn from(value: u8) -> Self {
        match value {
            0x00 => DtcFormatIdentifier::SaeJ2012DaDtcFormat00,
            0x01 => DtcFormatIdentifier::Iso14229_1DtcFormat,
            0x02 => DtcFormatIdentifier::SaeJ1939_73DtcFormat,
            0x03 => DtcFormatIdentifier::Iso11992_4DtcFormat,
            0x04 => DtcFormatIdentifier::SaeJ2012DaDtcFormat04,
            val => DtcFormatIdentifier::IsoSaeReserved(val),
        }
    }
}

impl DtcFormatIdentifier {
    /// The raw format-identifier byte.
    ///
    /// `const`, unlike `u8::from(identifier)`, so a `const` table can read it back out.
    #[must_use]
    pub const fn value(&self) -> u8 {
        match self {
            Self::SaeJ2012DaDtcFormat00 => 0x00,
            Self::Iso14229_1DtcFormat => 0x01,
            Self::SaeJ1939_73DtcFormat => 0x02,
            Self::Iso11992_4DtcFormat => 0x03,
            Self::SaeJ2012DaDtcFormat04 => 0x04,
            Self::IsoSaeReserved(value) => *value,
        }
    }
}

impl From<DtcFormatIdentifier> for u8 {
    fn from(val: DtcFormatIdentifier) -> Self {
        match val {
            DtcFormatIdentifier::SaeJ2012DaDtcFormat00 => 0x00,
            DtcFormatIdentifier::Iso14229_1DtcFormat => 0x01,
            DtcFormatIdentifier::SaeJ1939_73DtcFormat => 0x02,
            DtcFormatIdentifier::Iso11992_4DtcFormat => 0x03,
            DtcFormatIdentifier::SaeJ2012DaDtcFormat04 => 0x04,
            DtcFormatIdentifier::IsoSaeReserved(value) => value, // Default value for reserved
        }
    }
}

/// Use to clear all DTCs in a [`crate::ClearDiagnosticInfoRequest`]
pub const CLEAR_ALL_DTCS: DtcRecord = DtcRecord {
    high_byte: 0xFF,
    middle_byte: 0xFF,
    low_byte: 0xFF,
};

/// A 3-byte Diagnostic Trouble Code number (high, middle, low bytes).
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DtcRecord {
    high_byte: u8,
    middle_byte: u8,
    low_byte: u8,
}

impl DtcRecord {
    /// Create a `DtcRecord` from its three component bytes.
    #[must_use]
    pub const fn new(high_byte: u8, middle_byte: u8, low_byte: u8) -> Self {
        Self {
            high_byte,
            middle_byte,
            low_byte,
        }
    }

    /// The most significant of the three DTC bytes.
    ///
    /// What it means depends on the server's [`DtcFormatIdentifier`]: ISO 14229-1 itself
    /// specifies no decoding method for the three bytes (clause 12.3.2.3), deferring to
    /// whichever of SAE J2012-DA, ISO 11992-4, SAE J1939-73 or ISO 15031-6 the format
    /// identifier names.
    ///
    /// Annex D.1 Table D.1 does assign meaning, but to whole 3-byte `groupOfDTC` *values*, not
    /// to this byte in isolation — and its powertrain/chassis/body/network rows are explicitly
    /// "to be determined by vehicle manufacturer". The one byte-level assignment it makes is to
    /// the *low* byte: for `0xFFFF00`-`0xFFFFFE` that byte is a
    /// [`FunctionalGroupIdentifier`](crate::FunctionalGroupIdentifier).
    #[must_use]
    pub const fn high_byte(&self) -> u8 {
        self.high_byte
    }

    /// The middle of the three DTC bytes. See [`high_byte`](Self::high_byte) for why this
    /// crate does not ascribe a meaning to it.
    #[must_use]
    pub const fn middle_byte(&self) -> u8 {
        self.middle_byte
    }

    /// The least significant of the three DTC bytes.
    ///
    /// In SAE J2012-DA format this is the failure type byte, and for a `groupOfDTC` in
    /// `0xFFFF00`-`0xFFFFFE` it is a
    /// [`FunctionalGroupIdentifier`](crate::FunctionalGroupIdentifier) (ISO 14229-1:2020 Annex
    /// D.1 Table D.1). Neither reading is universal — see [`high_byte`](Self::high_byte).
    #[must_use]
    pub const fn low_byte(&self) -> u8 {
        self.low_byte
    }
}

impl TryFrom<u32> for DtcRecord {
    type Error = Error;

    /// A DTC is three bytes, so only the low 24 bits of a `u32` are a valid DTC.
    ///
    /// This is `TryFrom` rather than `From` because masking the top byte away silently would
    /// make `0xFF01_0203` and `0x0001_0203` the same record — a caller who has a DTC in a `u32`
    /// from elsewhere and one byte too many would get a wrong DTC with no signal.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDtcRecord`] if `value` exceeds `0x00FF_FFFF`.
    fn try_from(value: u32) -> Result<Self, Error> {
        if value > 0x00FF_FFFF {
            return Err(Error::InvalidDtcRecord(value));
        }
        #[allow(clippy::cast_possible_truncation)]
        Ok(Self {
            high_byte: ((value >> 16) & 0xFF) as u8,
            middle_byte: ((value >> 8) & 0xFF) as u8,
            low_byte: (value & 0xFF) as u8,
        })
    }
}

impl DtcRecord {
    /// The three DTC bytes as the low 24 bits of a `u32`, big-endian.
    ///
    /// `const`, unlike `u32::from(record)`: trait methods are not callable in a `const fn` on
    /// stable, so a `const` table of DTC values needs this.
    #[must_use]
    pub const fn to_u32(&self) -> u32 {
        ((self.high_byte as u32) << 16) | ((self.middle_byte as u32) << 8) | self.low_byte as u32
    }
}

impl From<DtcRecord> for u32 {
    fn from(value: DtcRecord) -> Self {
        (u32::from(value.high_byte) << 16)
            | (u32::from(value.middle_byte) << 8)
            | u32::from(value.low_byte)
    }
}

impl Encode for DtcRecord {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_all(writer, &[self.high_byte, self.middle_byte, self.low_byte]).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for DtcRecord {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 3 {
            return Err(Error::InsufficientData(Incomplete {
                needed: 3,
                available: buf.len(),
            }));
        }
        Ok((
            Self {
                high_byte: buf[0],
                middle_byte: buf[1],
                low_byte: buf[2],
            },
            &buf[3..],
        ))
    }
}

impl<'a> DecodeIter<'a> for DtcRecord {
    type Error = crate::Error;
    const WIRE_SIZE: Option<usize> = Some(3);

    fn decode_next(buf: &'a [u8]) -> Result<Option<(Self, &'a [u8])>, Error> {
        if buf.is_empty() {
            return Ok(None);
        }
        Decode::decode(buf).map(Some)
    }
}

/// Used to distinguish commands sent by the test equipment between different functional system groups
/// within an electrical architecture which consists of many different servers.
///
/// For the purpose of:
///     * Requesting DTC status from a vehicle
///     * Clearing DTC information in the vehicle
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FunctionalGroupIdentifier {
    /// 0x00 to 0x32
    /// 0x34 to 0xCF
    /// 0xE0 to 0xFD
    /// 0xFF
    IsoSaeReserved(u8),
    /// 0x33
    EmissionsSystemGroup,
    /// 0xD0
    SafetySystemGroup,

    /// 0xD1 to 0xDF
    /// For future use
    LegislativeSystemGroup(u8),

    /// 0xFE
    VobdSystem,
}

impl FunctionalGroupIdentifier {
    /// Return the raw `u8` value of this functional group identifier.
    #[must_use]
    pub const fn value(&self) -> u8 {
        match self {
            FunctionalGroupIdentifier::EmissionsSystemGroup => 0x33,
            FunctionalGroupIdentifier::SafetySystemGroup => 0xD0,
            FunctionalGroupIdentifier::VobdSystem => 0xFE,
            FunctionalGroupIdentifier::LegislativeSystemGroup(value)
            | FunctionalGroupIdentifier::IsoSaeReserved(value) => *value,
        }
    }
}

impl From<u8> for FunctionalGroupIdentifier {
    fn from(value: u8) -> Self {
        match value {
            0x33 => FunctionalGroupIdentifier::EmissionsSystemGroup,
            0xD0 => FunctionalGroupIdentifier::SafetySystemGroup,
            0xFE => FunctionalGroupIdentifier::VobdSystem,
            0xD1..=0xDF => FunctionalGroupIdentifier::LegislativeSystemGroup(value),
            _ => FunctionalGroupIdentifier::IsoSaeReserved(value),
        }
    }
}

impl From<FunctionalGroupIdentifier> for u8 {
    fn from(value: FunctionalGroupIdentifier) -> Self {
        value.value()
    }
}

impl Encode for FunctionalGroupIdentifier {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, self.value()).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for FunctionalGroupIdentifier {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}

/// GTR DTC Class Information
///
/// Bits 7-5 of the DtcSeverityMask/DTCSeverity parameters contain severity information (optional)
/// Bits 4-0 of the DtcSeverityMask/DTCSeverity parameters contain class information (mandatory)
///
/// DTCCLASS_
#[bitmask(u8)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum DtcSeverityMask {
    // GtrDtcClassInfo
    /// Unclassified
    DtcClass0,

    /// Matches GTR module B Class A definition
    /// Malfunction is Class A when On-Board Diagnostic (OBD) threshold limits (OTL) are assumed to be exceeded
    /// It is accepted that the emissions may not be above the OTLs when this class of malfunction occurs
    DtcClass1,

    /// Matches GTR module B Class B1 definition
    DtcClass2,
    /// Matches GTR module B Class B2 definition
    DtcClass3,
    /// Matches GTR module B Class C definition
    DtcClass4,

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

impl DtcSeverityMask {
    /// Returns `true` if at least one DTC class bit (bits 0-4) is set.
    /// Multiple class bits may be set to query multiple DTC classes at once.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.intersects(
            Self::DtcClass0 | Self::DtcClass1 | Self::DtcClass2 | Self::DtcClass3 | Self::DtcClass4,
        )
    }
}

impl Encode for DtcSeverityMask {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, self.bits()).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for DtcSeverityMask {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}

/// Identifies which `DTCStoredDataRecord` is being requested.
///
/// Setting to `0xFF` will return all `DTCStoredDataRecords` at once.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DtcStoredDataRecordNumber(u8);

impl DtcStoredDataRecordNumber {
    /// Create a `DtcStoredDataRecordNumber` from a raw byte. Every byte is accepted.
    ///
    /// Total, like [`DtcSnapshotRecordNumber::new`](crate::DtcSnapshotRecordNumber::new) and
    /// [`DtcExtDataRecordNumber::new`](crate::DtcExtDataRecordNumber::new), because decoding is
    /// deliberately liberal and `From<u8>` already accepted anything — so a fallible `new`
    /// promised a guarantee the type did not actually hold. Use
    /// [`is_reserved`](Self::is_reserved) when you need the check.
    ///
    /// Clause 12.3.3.2 reserves `0x00` for legislated purposes, makes `0x01`-`0xFE` available
    /// for vehicle-manufacturer use, and gives `0xFF` the meaning "report all stored records".
    /// Note that `0xF0` is *not* reserved here — that belongs to the
    /// [`DtcSnapshotRecordNumber`](crate::DtcSnapshotRecordNumber) space, which the spec says
    /// does not share an address space with this one.
    #[must_use]
    pub const fn new(record_number: u8) -> Self {
        Self(record_number)
    }

    /// Whether this record number is the `0x00` that clause 12.3.3.2 reserves for legislated
    /// purposes, and which a client therefore should not request.
    #[must_use]
    pub const fn is_reserved(&self) -> bool {
        self.0 == 0
    }

    /// Return the raw record-number byte.
    ///
    /// May be the reserved `0x00`; check [`is_reserved`](Self::is_reserved) if that matters.
    #[must_use]
    pub const fn value(&self) -> u8 {
        self.0
    }
}

impl PartialEq<u8> for DtcStoredDataRecordNumber {
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other
    }
}

impl From<u8> for DtcStoredDataRecordNumber {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Encode for DtcStoredDataRecordNumber {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, self.0).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for DtcStoredDataRecordNumber {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}

#[cfg(test)]
mod encode_param_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_stored_data_record_number() {
        let n = DtcStoredDataRecordNumber::new(0x05);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x05);
        assert_encode_size_agrees(&n);
    }

    #[test]
    fn stored_data_record_number_accepts_every_byte_and_flags_the_reserved_one() {
        // ISO 14229-1:2020 clause 12.3.3.2 reserves only 0x00 for this parameter: "DTCStoredData
        // records in range of 0x01 through 0xFE shall be available for vehicle manufacturer
        // specific usage", and 0xFF requests all records. 0xF0 belongs to the *snapshot*
        // record-number space and used to be rejected here by a check copied from there.
        for byte in [0x00u8, 0x01, 0xF0, 0xFE, 0xFF] {
            let number = DtcStoredDataRecordNumber::new(byte);
            assert_eq!(number.value(), byte);
            assert_eq!(number, byte, "PartialEq<u8> must agree with value()");
            assert_eq!(number, DtcStoredDataRecordNumber::from(byte));
            assert_eq!(number.is_reserved(), byte == 0x00, "for {byte:#04X}");
        }

        // Decoding is liberal for the same reason `new` is total: a response from a foreign
        // implementation must still be inspectable.
        let (decoded, _) = <DtcStoredDataRecordNumber as Decode>::decode(&[0x00]).unwrap();
        assert!(decoded.is_reserved());
    }

    #[test]
    fn encode_severity_mask() {
        let m = DtcSeverityMask::CheckImmediately;
        let mut buf = [0u8; 4];
        let written = Encode::encode(&m, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0b1000_0000);
        assert_encode_size_agrees(&m);
    }

    #[test]
    fn encode_functional_group_identifier_named() {
        let g = FunctionalGroupIdentifier::EmissionsSystemGroup;
        let mut buf = [0u8; 4];
        let written = Encode::encode(&g, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x33);
        assert_encode_size_agrees(&g);
    }

    #[test]
    fn functional_group_identifier_value_does_not_panic_on_reserved() {
        // Regression: value() previously called todo!() for carried-byte variants.
        let g = FunctionalGroupIdentifier::from(0x10); // -> IsoSaeReserved(0x10)
        assert_eq!(g.value(), 0x10);
        let g2 = FunctionalGroupIdentifier::from(0xD5); // -> LegislativeSystemGroup(0xD5)
        assert_eq!(g2.value(), 0xD5);
    }
}

#[cfg(test)]
mod dtc_status_tests {
    use super::*;

    #[test]
    fn dtc_record_exposes_its_three_wire_bytes() {
        // A decoded DtcRecord has to be inspectable byte-wise: D.1 assigns meaning to the
        // high byte (system group) separately from the middle and low bytes.
        let record = DtcRecord::try_from(0x12_3456).unwrap();
        assert_eq!(record.high_byte(), 0x12);
        assert_eq!(record.middle_byte(), 0x34);
        assert_eq!(record.low_byte(), 0x56);
    }

    #[test]
    fn dtc_record_byte_accessors_are_usable_in_const_context() {
        const HIGH: u8 = CLEAR_ALL_DTCS.high_byte();
        assert_eq!(HIGH, 0xFF);
    }

    #[test]
    fn status_mask() {
        let status_mask = DtcStatusMask::TestFailed | DtcStatusMask::PendingDtc;
        assert_eq!(status_mask.bits(), 0b0000_0101);

        let status_mask = DtcStatusMask::TestFailedThisOperationCycle
            | DtcStatusMask::TestNotCompletedSinceLastClear;

        assert_eq!(status_mask.bits(), 0b0001_0010);
    }

    #[test]
    fn gtr_dtc_class_info() {
        let dtc_class = DtcSeverityMask::DtcClass1 | DtcSeverityMask::MaintenanceOnly;
        assert_eq!(dtc_class.bits(), 0b0010_0010);
        assert!(dtc_class.is_valid());
    }

    #[test]
    fn dtc_severity_info() {
        let dtc_severity = DtcSeverityMask::CheckImmediately;
        assert_eq!(dtc_severity.bits(), 0b1000_0000);
    }

    #[test]
    fn dtc_record_encode_decode() {
        let record = DtcRecord::new(0x01, 0x02, 0x03);
        let mut buf = [0u8; 3];
        let written = Encode::encode(&record, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 3);
        let (decoded, rest) = <DtcRecord as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, record);
        assert!(rest.is_empty());
    }
}
