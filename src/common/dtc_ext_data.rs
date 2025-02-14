use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    DTCRecord, DTCStatusMask, Error, IterableWireFormat, SingleValueWireFormat, WireFormat,
};

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
    /// Either the echo of the DTCExtDataRecordNumber parameter specified by the client in the
    /// reportDTCExtDataRecordByDTCNumber, reportDTCExtendedDataRecordIdentification or
    /// reportDTCExtDataRecordByRecordNumber request, or the actual DTCExtDataRecordNumber of a stored DTCExtendedData record.
    pub record_number: DTCExtDataRecordNumber,

    pub data: Vec<UserPayload>,
}

impl<UserPayload: IterableWireFormat> WireFormat for DTCExtDataRecord<UserPayload> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let record_number = reader.read_u8();
        if record_number.is_err() {
            return Ok(None);
        }

        // DTCExtData record number
        let record_number = DTCExtDataRecordNumber::new(record_number.unwrap());

        let mut data = Vec::new();
        while let Ok(Some(payload)) = UserPayload::option_from_reader(reader) {
            data.push(payload);
        }

        Ok(Some(Self {
            record_number,
            data,
        }))
    }

    fn required_size(&self) -> usize {
        // 1 byte for record number
        // n bytes of data per UserPayload
        1 + self.data.iter().map(|d| d.required_size()).sum::<usize>()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.record_number.value())?;
        for d in &self.data {
            d.to_writer(writer)?;
        }
        Ok(self.required_size())
    }
}

impl<UserPayload: IterableWireFormat> IterableWireFormat for DTCExtDataRecord<UserPayload> {}

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
        for result in DTCExtDataRecord::from_reader_iterable(reader) {
            match result {
                Ok(record) => record_data.push(record),
                Err(e) => return Err(e),
            }
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
