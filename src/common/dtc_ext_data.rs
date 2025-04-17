use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    DTCRecord, DTCStatusMask, Error, IterableWireFormat, SingleValueWireFormat, WireFormat,
};

use super::{DTCSeverityMask, FunctionalGroupIdentifier};

/// The DTCExtDataRecordNumber is used in the request message to get a stored [DTCExtDataRecord]
/// Its used to specify the type of DTCExtDataRecord to be reported.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DTCExtDataRecordNumber {
    // 0x00, 0xF0-0xFD are reserved
    ISOSAEReserved(u8),

    /// Vehicle manufactured specific stored [DTCExtDataRecord]s
    ///
    /// 0x01-0x8F
    VehicleManufacturer(u8),

    /// Requests the server to report regulated emissions OBD stored DTCExtendedDataRecords.
    /// The values are specified in SAE J1979-DA.
    ///
    /// 0x90-0x9F
    RegulatedEmissionsOBDDTCExtDataRecords(u8),

    /// The DTCExtDataRecordNumber parameter is used to specify the DTC number of the DTCExtendedData record to be reported.
    ///
    /// 0xA0-0xEF
    RegulatedDTCExtDataRecords(u8),

    /// Requests the server to report all regulated emissions OBD stored DTCExtendedDataRecords.
    AllRegulatedEmissionsOBDDTCExtDataRecords,

    /// Requests the server to report all stored DTCExtendedDataRecords
    AllDTCExtDataRecords,
}

impl DTCExtDataRecordNumber {
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

impl WireFormat for DTCExtDataRecordNumber {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        Ok(Some(Self::new(reader.read_u8()?)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.value())?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for DTCExtDataRecordNumber {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DTCExtDataRecord<UserPayload> {
    pub data: Vec<UserPayload>,
}

impl<UserPayload: IterableWireFormat> WireFormat for DTCExtDataRecord<UserPayload> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut data = Vec::new();
        for payload in UserPayload::from_reader_iterable(reader) {
            match payload {
                Err(_) => return Ok(None),
                Ok(payload) => {
                    data.push(payload);
                }
            }
        }

        Ok(Some(Self { data }))
    }

    fn required_size(&self) -> usize {
        // n bytes of data per UserPayload
        self.data.iter().map(|d| d.required_size()).sum::<usize>()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        for d in &self.data {
            d.to_writer(writer)?;
        }
        Ok(self.required_size())
    }
}

impl<UserPayload: IterableWireFormat> SingleValueWireFormat for DTCExtDataRecord<UserPayload> {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DTCExtDataRecordList<UserPayload> {
    pub mask_record: DTCRecord,
    pub status_mask: DTCStatusMask,
    pub record_data: Vec<DTCExtDataRecord<UserPayload>>,
}

impl<UserPayload: IterableWireFormat> WireFormat for DTCExtDataRecordList<UserPayload> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mask_record = DTCRecord::from_reader(reader)?;
        let status_mask = DTCStatusMask::from_reader(reader)?;
        let mut record_data = Vec::new();
        // Read the record number, and then the payload
        if let Some(record) = DTCExtDataRecord::option_from_reader(reader)? {
            record_data.push(record);
        }
        Ok(Some(Self {
            mask_record,
            status_mask,
            record_data,
        }))
    }

    fn required_size(&self) -> usize {
        self.mask_record.required_size()
            + self.status_mask.required_size()
            + self
                .record_data
                .iter()
                .map(|r| r.required_size())
                .sum::<usize>()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.mask_record.to_writer(writer)?;
        self.status_mask.to_writer(writer)?;
        for record in &self.record_data {
            record.to_writer(writer)?;
        }
        Ok(self.required_size())
    }
}

impl<UserPayload: IterableWireFormat> SingleValueWireFormat for DTCExtDataRecordList<UserPayload> {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Represents a record containing information about the severity of a Diagnostic Trouble Code (DTC).
///
/// # Fields
/// - `severity`: The severity mask associated with the DTC, indicating the level of severity.
/// - `functional_group_identifier`: Identifier for the functional group associated with the DTC.
/// - `dtc_record`: The actual DTC record containing diagnostic information.
/// - `dtc_status_mask`: The status mask of the DTC, representing its current state.
pub struct DTCSeverityRecord {
    pub severity: DTCSeverityMask,
    pub functional_group_identifier: FunctionalGroupIdentifier,
    pub dtc_record: DTCRecord,
    pub dtc_status_mask: DTCStatusMask,
}

/// Implementation of the `WireFormat` trait for the `DTCSeverityRecord` struct.
///
/// This implementation provides methods for reading and writing `DTCSeverityRecord`
/// instances to and from a binary format, as well as calculating the required size
/// for serialization.
///
/// # Methods
///
/// - `option_from_reader`:
///   Reads a `DTCSeverityRecord` from a reader. If the reader encounters an error
///   while reading the severity byte, it returns `Ok(None)`. Otherwise, it constructs
///   a `DTCSeverityRecord` from the binary data.
///
///   ## Parameters:
///   - `reader`: A mutable reference to an object implementing the `std::io::Read` trait.
///
///   ## Returns:
///   - `Result<Option<Self>, Error>`: Returns `Ok(Some(Self))` if successful, `Ok(None)`
///     if the severity byte cannot be read, or an error if any other issue occurs.
///
/// - `required_size`:
///   Returns the size in bytes required to serialize the `DTCSeverityRecord`.
///
///   ## Returns:
///   - `usize`: The size in bytes (always 6 for this implementation).
///
/// - `to_writer`:
///   Writes the `DTCSeverityRecord` to a writer in binary format.
///
///   ## Parameters:
///   - `writer`: A mutable reference to an object implementing the `std::io::Write` trait.
///
///   ## Returns:
///   - `Result<usize, Error>`: Returns the number of bytes written (always 6 for this
///     implementation) or an error if writing fails.
impl WireFormat for DTCSeverityRecord {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let sev = match reader.read_u8() {
            Ok(sev) => sev,
            Err(_) => return Ok(None),
        };

        let severity = DTCSeverityMask::from(sev);
        let functional_group_identifier = FunctionalGroupIdentifier::from(reader.read_u8()?);
        let dtc_record = DTCRecord::option_from_reader(reader)?.unwrap();
        let dtc_status_mask = DTCStatusMask::from(reader.read_u8()?);

        Ok(Some(Self {
            severity,
            functional_group_identifier,
            dtc_record,
            dtc_status_mask,
        }))
    }

    fn required_size(&self) -> usize {
        6
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.severity.bits())?;
        writer.write_u8(self.functional_group_identifier.value())?;
        self.dtc_record.to_writer(writer)?;
        self.dtc_status_mask.to_writer(writer)?;
        Ok(self.required_size())
    }
}

/// Implements the `IterableWireFormat` trait for the `DTCSeverityRecord` type.
///
/// This allows `DTCSeverityRecord` to be serialized and deserialized in a format
/// suitable for wire transmission, enabling iteration over its data structure
/// in a protocol-compliant manner.
impl IterableWireFormat for DTCSeverityRecord {}

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
}
