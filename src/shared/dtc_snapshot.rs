//! Diagnostic Trouble Code (DTC) Snapshot Data
//! Snapshot data represents a collection of sensor values captured when a DTC is triggered.
//! Represents the state of the server at the time the DTC was triggered.

use crate::{Decode, Encode, Error};

/// Identifies which DTC snapshot record is being requested or reported.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DTCSnapshotRecordNumber {
    /// Reserved for Legislative purposes
    Reserved(u8),
    /// Indicates the number of the specific `DTCSnapshot` data record requested
    Number(u8),
    /// Requests that the server report all `DTCSnapshot` data records at once
    All,
}

impl DTCSnapshotRecordNumber {
    /// Create a new `DTCSnapshotRecordNumber` validating that it is in the range we expect
    #[must_use]
    pub fn new(record_number: u8) -> Self {
        match record_number {
            0x00 | 0xF0 => Self::Reserved(record_number),
            0xFF => Self::All,
            _ => Self::Number(record_number),
        }
    }
    /// Return the raw `u8` value of this snapshot record number.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn value(&self) -> u8 {
        match self {
            DTCSnapshotRecordNumber::Reserved(value) => *value,
            DTCSnapshotRecordNumber::Number(value) => *value,
            DTCSnapshotRecordNumber::All => 0xFF,
        }
    }
}

impl PartialEq<u8> for DTCSnapshotRecordNumber {
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other
    }
}

impl Encode for DTCSnapshotRecordNumber {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for DTCSnapshotRecordNumber {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((Self::new(buf[0]), &buf[1..]))
    }
}

#[cfg(test)]
mod snapshot {
    use super::*;

    #[test]
    fn snapshot_record_number() {
        let record = DTCSnapshotRecordNumber::new(0x01);
        assert_eq!(record.value(), 0x01);
        assert_eq!(record, DTCSnapshotRecordNumber::Number(0x01));

        let all = DTCSnapshotRecordNumber::new(0xFF);
        assert_eq!(all, DTCSnapshotRecordNumber::All);
    }

    #[test]
    fn encode_snapshot_record_number() {
        use crate::test_util::assert_encode_size_agrees;
        let n = DTCSnapshotRecordNumber::new(0x02);
        let mut buf = [0u8; 4];
        let written = crate::Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x02);
        assert_encode_size_agrees(&n);
    }
}
