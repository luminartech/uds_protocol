use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::{
    DTCRecord, DTCStatusMask, Error, IterableWireFormat, SingleValueWireFormat, WireFormat,
};

/// The `DTCExtDataRecordNumber` is used in the request message to get a stored [`DTCExtDataRecord`]
/// Its used to specify the type of `DTCExtDataRecord` to be reported.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DTCExtDataRecordNumber {
    // 0x00, 0xF0-0xFD are reserved
    ISOSAEReserved(u8),

    /// Vehicle manufactured specific stored [`DTCExtDataRecord`]s
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
    AllRegulatedEmissionsOBDDTCExtDataRecords =
        Self::ALL_REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS,

    /// Requests the server to report all stored `DTCExtendedDataRecords`
    AllDTCExtDataRecords = Self::ALL_DTC_EXT_DATA_RECORDS,
}

impl DTCExtDataRecordNumber {
    pub const ISO_SAE_RESERVED: u8 = 0x00;
    pub const ISO_SAE_RESERVED_START: u8 = 0xF0;
    pub const ISO_SAE_RESERVED_END: u8 = 0xFD;
    pub const VEHICLE_MANUFACTURER_START: u8 = 0x01;
    pub const VEHICLE_MANUFACTURER_END: u8 = 0x8F;
    pub const REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS_START: u8 = 0x90;
    pub const REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS_END: u8 = 0x9F;
    pub const REGULATED_DTC_EXT_DATA_RECORDS_START: u8 = 0xA0;
    pub const REGULATED_DTC_EXT_DATA_RECORDS_END: u8 = 0xEF;
    pub const ALL_REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS: u8 = 0xFE;
    pub const ALL_DTC_EXT_DATA_RECORDS: u8 = 0xFF;

    #[must_use]
    pub fn new(value: u8) -> Self {
        match value {
            DTCExtDataRecordNumber::ISO_SAE_RESERVED
            | DTCExtDataRecordNumber::ISO_SAE_RESERVED_START
                ..=DTCExtDataRecordNumber::ISO_SAE_RESERVED_END => {
                DTCExtDataRecordNumber::ISOSAEReserved(value)
            }
            DTCExtDataRecordNumber::VEHICLE_MANUFACTURER_START
                ..=DTCExtDataRecordNumber::VEHICLE_MANUFACTURER_END => {
                DTCExtDataRecordNumber::VehicleManufacturer(value)
            }
            DTCExtDataRecordNumber::REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS_START
                ..=DTCExtDataRecordNumber::REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS_END => {
                DTCExtDataRecordNumber::RegulatedEmissionsOBDDTCExtDataRecords(value)
            }
            DTCExtDataRecordNumber::REGULATED_DTC_EXT_DATA_RECORDS_START
                ..=DTCExtDataRecordNumber::REGULATED_DTC_EXT_DATA_RECORDS_END => {
                DTCExtDataRecordNumber::RegulatedDTCExtDataRecords(value)
            }
            DTCExtDataRecordNumber::ALL_REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS => {
                DTCExtDataRecordNumber::AllRegulatedEmissionsOBDDTCExtDataRecords
            }
            DTCExtDataRecordNumber::ALL_DTC_EXT_DATA_RECORDS => {
                DTCExtDataRecordNumber::AllDTCExtDataRecords
            }
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn value(&self) -> u8 {
        match self {
            Self::ISOSAEReserved(value) => *value,
            Self::VehicleManufacturer(value) => *value,
            Self::RegulatedEmissionsOBDDTCExtDataRecords(value) => *value,
            Self::RegulatedDTCExtDataRecords(value) => *value,
            Self::AllRegulatedEmissionsOBDDTCExtDataRecords => {
                DTCExtDataRecordNumber::ALL_REGULATED_EMISSIONS_OBD_DTC_EXT_DATA_RECORDS
            }
            Self::AllDTCExtDataRecords => DTCExtDataRecordNumber::ALL_DTC_EXT_DATA_RECORDS,
        }
    }
}

impl PartialEq<u8> for DTCExtDataRecordNumber {
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other
    }
}

impl WireFormat for DTCExtDataRecordNumber {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        match reader.read_u8() {
            Ok(ext_data_record_number) => Ok(Some(Self::new(ext_data_record_number))),
            Err(_) => Ok(None),
        }
    }

    fn required_size(&self) -> usize {
        1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.value())?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for DTCExtDataRecordNumber {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
pub struct DTCExtDataRecord<UserPayload> {
    pub data: Vec<UserPayload>,
}

impl<UserPayload: IterableWireFormat> WireFormat for DTCExtDataRecord<UserPayload> {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut data = Vec::new();
        for payload in UserPayload::decode_iterable(reader) {
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
        self.data
            .iter()
            .map(WireFormat::required_size)
            .sum::<usize>()
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        for d in &self.data {
            d.encode(writer)?;
        }
        Ok(self.required_size())
    }
}

impl<UserPayload: IterableWireFormat> SingleValueWireFormat for DTCExtDataRecord<UserPayload> {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
pub struct DTCExtDataRecordList<UserPayload> {
    pub mask_record: DTCRecord,
    pub status_mask: DTCStatusMask,
    pub record_data: Vec<DTCExtDataRecord<UserPayload>>,
}

impl<UserPayload: IterableWireFormat> WireFormat for DTCExtDataRecordList<UserPayload> {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mask_record = DTCRecord::decode_single_value(reader)?;
        let status_mask = DTCStatusMask::decode_single_value(reader)?;
        let mut record_data = Vec::new();
        // Read the record number, and then the payload
        if let Some(record) = DTCExtDataRecord::decode(reader)? {
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
                .map(WireFormat::required_size)
                .sum::<usize>()
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.mask_record.encode(writer)?;
        self.status_mask.encode(writer)?;
        for record in &self.record_data {
            record.encode(writer)?;
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
