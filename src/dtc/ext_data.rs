use crate::{Decode, Encode, Error, Incomplete};
use automotive_wire_codec::write_u8;

/// The `DtcExtDataRecordNumber` is used in the request message to get a stored `DTCExtDataRecord`
/// It's used to specify the type of `DTCExtDataRecord` to be reported.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DtcExtDataRecordNumber {
    /// ISO/SAE reserved record numbers (`0x00`, `0xF0-0xFD`).
    IsoSaeReserved(u8),

    /// Vehicle manufacturer-specific stored `DTCExtDataRecord`s
    ///
    /// 0x01-0x8F
    VehicleManufacturer(u8),

    /// Requests the server to report regulated emissions OBD stored `DTCExtendedDataRecords`.
    /// The values are specified in SAE J1979-DA.
    ///
    /// 0x90-0x9F
    RegulatedEmissionsObdDtcExtDataRecords(u8),

    /// The `DtcExtDataRecordNumber` parameter is used to specify the DTC number of the `DTCExtendedData` record to be reported.
    ///
    /// 0xA0-0xEF
    RegulatedDtcExtDataRecords(u8),

    /// Requests the server to report all regulated emissions OBD stored `DTCExtendedDataRecords`.
    AllRegulatedEmissionsObdDtcExtDataRecords,

    /// Requests the server to report all stored `DTCExtendedDataRecords`
    AllDtcExtDataRecords,
}

impl DtcExtDataRecordNumber {
    /// Create a new `DtcExtDataRecordNumber` from a raw byte, mapping it to the correct variant.
    #[must_use]
    pub fn new(value: u8) -> Self {
        match value {
            0x00 | 0xF0..=0xFD => Self::IsoSaeReserved(value),
            0x01..=0x8F => Self::VehicleManufacturer(value),
            0x90..=0x9F => Self::RegulatedEmissionsObdDtcExtDataRecords(value),
            0xA0..=0xEF => Self::RegulatedDtcExtDataRecords(value),
            0xFE => Self::AllRegulatedEmissionsObdDtcExtDataRecords,
            0xFF => Self::AllDtcExtDataRecords,
        }
    }

    /// Return the raw `u8` value of this record number.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub const fn value(&self) -> u8 {
        match self {
            Self::IsoSaeReserved(value) => *value,
            Self::VehicleManufacturer(value) => *value,
            Self::RegulatedEmissionsObdDtcExtDataRecords(value) => *value,
            Self::RegulatedDtcExtDataRecords(value) => *value,
            Self::AllRegulatedEmissionsObdDtcExtDataRecords => 0xFE,
            Self::AllDtcExtDataRecords => 0xFF,
        }
    }
}

impl PartialEq<u8> for DtcExtDataRecordNumber {
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other
    }
}

impl Encode for DtcExtDataRecordNumber {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, self.value()).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for DtcExtDataRecordNumber {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        Ok((Self::new(buf[0]), &buf[1..]))
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_number() {
        let record_number = DtcExtDataRecordNumber::new(0x00);
        assert_eq!(record_number, DtcExtDataRecordNumber::IsoSaeReserved(0x00));
        assert_eq!(record_number.value(), 0x00);
    }

    #[test]
    fn encode_ext_data_record_number() {
        use crate::test_util::assert_encode_size_agrees;
        let n = DtcExtDataRecordNumber::new(0x90);
        let mut buf = [0u8; 4];
        let written = crate::Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x90);
        assert_encode_size_agrees(&n);
    }
}
