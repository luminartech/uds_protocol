use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{Error, SingleValueWireFormat, WireFormat};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DTCExtDataRecordNumber {
    // 0x00, 0xF0-0xFD are reserved
    ISOSAEReserved(u8),

    /// Vehicle manufactured specific stored [DTCExtDataRecord]s
    /// 0x01-0x8F
    VehicleManufacturer(u8),

    /// Requests the server to report regulated emissions OBD stored DTCExtendedDataRecords.
    /// The values are specified in SAE J1979-DA.
    /// 0x90-0x9F
    RegulatedEmissionsOBDDTCExtDataRecords(u8),

    /// The DTCExtDataRecordNumber parameter is used to specify the DTC number of the DTCExtendedData record to be reported.
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
pub struct DTCExtDataRecord {
    /// Either the echo of the DTCExtDataRecordNumber parameter specified by the client in the
    /// reportDTCExtDataRecordByDTCNumber, reportDTCExtendedDataRecordIdentification or
    /// reportDTCExtDataRecordByRecordNumber request, or the actual DTCExtDataRecordNumber of a stored DTCExtendedData record.
    pub record_number: DTCExtDataRecordNumber,

    pub data: Vec<u8>,
}

impl WireFormat for DTCExtDataRecord {
    fn required_size(&self) -> usize {
        1 + self.data.len()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.record_number.value())?;
        writer.write_all(&self.data)?;
        Ok(self.required_size())
    }

    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let record_number = DTCExtDataRecordNumber::new(reader.read_u8()?);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(Self {
            record_number,
            data,
        }))
    }
}

impl SingleValueWireFormat for DTCExtDataRecord {}
