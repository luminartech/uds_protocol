use crate::{Decode, Encode, Error};

/// The `DTCExtDataRecordNumber` is used in the request message to get a stored `DTCExtDataRecord`
/// It's used to specify the type of `DTCExtDataRecord` to be reported.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DTCExtDataRecordNumber {
    /// ISO/SAE reserved record numbers (`0x00`, `0xF0-0xFD`).
    ISOSAEReserved(u8),

    /// Vehicle manufacturer-specific stored `DTCExtDataRecord`s
    ///
    /// 0x01-0x8F
    VehicleManufacturer(u8),

    /// Requests the server to report regulated emissions OBD stored `DTCExtendedDataRecords`.
    /// The values are specified in SAE J1979-DA.
    ///
    /// 0x90-0x9F
    RegulatedEmissionsOBDDTCExtDataRecords(u8),

    /// The `DTCExtDataRecordNumber` parameter is used to specify the DTC number of the `DTCExtendedData` record to be reported.
    ///
    /// 0xA0-0xEF
    RegulatedDTCExtDataRecords(u8),

    /// Requests the server to report all regulated emissions OBD stored `DTCExtendedDataRecords`.
    AllRegulatedEmissionsOBDDTCExtDataRecords,

    /// Requests the server to report all stored `DTCExtendedDataRecords`
    AllDTCExtDataRecords,
}

impl DTCExtDataRecordNumber {
    /// Create a new `DTCExtDataRecordNumber` from a raw byte, mapping it to the correct variant.
    #[must_use]
    pub fn new(value: u8) -> Self {
        match value {
            0x00 | 0xF0..=0xFD => Self::ISOSAEReserved(value),
            0x01..=0x8F => Self::VehicleManufacturer(value),
            0x90..=0x9F => Self::RegulatedEmissionsOBDDTCExtDataRecords(value),
            0xA0..=0xEF => Self::RegulatedDTCExtDataRecords(value),
            0xFE => Self::AllRegulatedEmissionsOBDDTCExtDataRecords,
            0xFF => Self::AllDTCExtDataRecords,
        }
    }

    /// Return the raw `u8` value of this record number.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn value(&self) -> u8 {
        match self {
            Self::ISOSAEReserved(value) => *value,
            Self::VehicleManufacturer(value) => *value,
            Self::RegulatedEmissionsOBDDTCExtDataRecords(value) => *value,
            Self::RegulatedDTCExtDataRecords(value) => *value,
            Self::AllRegulatedEmissionsOBDDTCExtDataRecords => 0xFE,
            Self::AllDTCExtDataRecords => 0xFF,
        }
    }
}

impl PartialEq<u8> for DTCExtDataRecordNumber {
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other
    }
}

impl Encode for DTCExtDataRecordNumber {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for DTCExtDataRecordNumber {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
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
        let record_number = DTCExtDataRecordNumber::new(0x00);
        assert_eq!(record_number, DTCExtDataRecordNumber::ISOSAEReserved(0x00));
        assert_eq!(record_number.value(), 0x00);
    }

    #[test]
    fn encode_ext_data_record_number() {
        use crate::test_util::assert_encode_size_agrees;
        let n = DTCExtDataRecordNumber::new(0x90);
        let mut buf = [0u8; 4];
        let written = crate::Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x90);
        assert_encode_size_agrees(&n);
    }
}
