//! Diagnostic Trouble Code (DTC) Snapshot Data
//! Snapshot data represents a collection of sensor values captured when a DTC is triggered.
//! Represents the state of the server at the time the DTC was triggered.
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    DTCMaskRecord, DTCStatusMask, Error, IterableWireFormat, SingleValueWireFormat, WireFormat,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DTCSnapshotRecordList<UserPayload> {
    pub mask_record: DTCMaskRecord,
    pub status_mask: DTCStatusMask,
    /// The number of the specific DTCSnapshot data record requested
    pub snapshot_data: Vec<(
        DTCSnapshotRecordNumber, 
        DTCSnapshotRecord<UserPayload>,
    )>,
}

impl<Identifier: IterableWireFormat> WireFormat for DTCSnapshotRecordList<Identifier> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mask_record = DTCMaskRecord::option_from_reader(reader)?;
        if mask_record.is_none() {
            return Ok(None);
        }
        let status_mask = DTCStatusMask::option_from_reader(reader)?;

        let mut snapshot_data = Vec::new();
        loop {
            let record_number = match DTCSnapshotRecordNumber::option_from_reader(reader) {
                Ok(Some(record_number)) => record_number,
                Ok(None) => break,
                Err(e) => return Err(e),
            };

            let record = match DTCSnapshotRecord::option_from_reader(reader) {
                Ok(Some(record)) => record,
                Ok(None) => break,
                Err(e) => return Err(e),
            };

            snapshot_data.push((record_number, record));
        }

        Ok(Some(Self {
            mask_record: mask_record.unwrap(),
            status_mask: status_mask.unwrap(),
            snapshot_data,
        }))
    }

    fn required_size(&self) -> usize {
        self.mask_record.required_size()
            + self.status_mask.required_size()
            + self
                .snapshot_data
                .iter()
                .fold(0, |acc, (record_number, record)| {
                    acc + record_number.required_size() + record.required_size()
                })
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.mask_record.to_writer(writer)?;
        self.status_mask.to_writer(writer)?;
        for (record_number, record) in &self.snapshot_data {
            record_number.to_writer(writer)?;
            record.to_writer(writer)?;
        }

        Ok(self.required_size())
    }
}

impl<UserPayload: IterableWireFormat> SingleValueWireFormat for DTCSnapshotRecordList<UserPayload> {}

/// Contains a snapshot of data values from the time of the system malfunction occurrence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DTCSnapshotRecord<UserPayload> {
    // pub record_number: u8,

    /// The data identifier (DID) for the data values taken at the time of the system malfunction occurrence
    /// These can be vehicle manufacturer specific
    /// See C.1 for broad categories of data identifiers
    /// The data values taken at the time of the system malfunction occurrence
    /// The data values are dependent on the data identifier, and are specified by the vehicle manufacturer/supplier
    pub data: Vec<UserPayload>,
}

impl<UserPayload: IterableWireFormat> WireFormat for DTCSnapshotRecord<UserPayload> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        // let record_number = reader.read_u8()?;
        let number_of_dids = reader.read_u8()?;
        let mut data = Vec::new();
        for payload in UserPayload::from_reader_iterable(reader) {
            match payload {
                Ok(did) => {
                    data.push(did);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        if number_of_dids != 0x00 && number_of_dids != data.len() as u8 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }

        Ok(Some(Self {
            data,
        }))
    }

    fn required_size(&self) -> usize {
        3 + self.data.iter().map(|d| d.required_size()).sum::<usize>()
    }

    // TODO: Must write the DIDs as well...
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // write 0x00 if the number of DIDs exceed 0xFF
        if self.data.len() > 0xFF {
            writer.write_u8(0)?;
        } else {
            writer.write_u8(self.data.len() as u8)?;
        }
        

        let mut payload_written = 0;
        for payload in &self.data {
            // Assumes this writes the DID as well, I think that's safe?
            payload_written += payload.to_writer(writer)?;
        }
        Ok(3 + payload_written)
    }
}

/// This might be a duplicate of the non-user defined DTC snapshot data
/// Indicates the number of the specific DTCSnapshot data record requested
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserDefDTCSnapshotRecordNumber(u8);

impl WireFormat for UserDefDTCSnapshotRecordNumber {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value = reader.read_u8()?;
        match value {
            // Reserved for Legislative purposes
            0x00 | 0xF0 => {
                return Err(Error::ReservedForLegislativeUse(
                    "UserDefDTCSnapshotRecordNumber".to_string(),
                    value,
                ))
            }
            // Requests that the server report all DTCSnapshot data records at once
            0xFF => {}
            _ => {}
        }
        Ok(Some(Self(value)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.0)?;
        Ok(1)
    }
}

impl SingleValueWireFormat for UserDefDTCSnapshotRecordNumber {}

impl From<u8> for UserDefDTCSnapshotRecordNumber {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct DTCSnapshotRecordNumber(u8);

impl DTCSnapshotRecordNumber {
    /// Create a new DTCSnapshotRecordNumber validating that it is in the range we expect
    pub fn new(record_number: u8) -> Result<Self, Error> {
        if record_number == 0 || record_number == 0xF0 {
            return Err(Error::ReservedForLegislativeUse(
                "DTCSnapshotRecordNumber".to_string(),
                record_number,
            ));
        }
        Ok(Self(record_number))
    }
}

impl WireFormat for DTCSnapshotRecordNumber {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value = match reader.read_u8() {
            Ok(byte) => byte,
            Err(_) => return Ok(None),
        };

        Ok(Some(Self(value)))
    }

    fn required_size(&self) -> usize {
        1
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.0)?;
        Ok(1)
    }
}

impl SingleValueWireFormat for DTCSnapshotRecordNumber {}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct DTCStoredDataList {
//     pub data: Vec<DTCStoredData>,
// }

#[cfg(test)]
mod snapshot {

    pub enum ProtocolPayload {
        Did4711([u8; 5]),
        Did8711([u8; 5]),
        
    }
    // Testing out a macro to make simplifying the enum to DID value "nicer"
    macro_rules! value_map {
        ($(($e:ident, $v:literal)),* $(,)?) => {
            pub fn value(&self) -> u16 {
            match self {
                $(ProtocolPayload::$e(_) => $v,)*
            }
        }
        }
    }
    impl ProtocolPayload {
        value_map![
            (Did4711, 0x4711),
            (Did8711, 0x8711),
        ];
    }


    impl WireFormat for ProtocolPayload {
        fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
            let mut identifier_data: [u8; 2] = [0; 2];
            match reader.read(&mut identifier_data)? {
                0 => return Ok(None),
                1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
                2 => (),
                _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
            };
            // read the identifier
            let identifier = u16::from_be_bytes(identifier_data);
            match identifier {
                0x4711 => {
                    let mut did_4711 = [0u8; 5];
                    match reader.read(&mut did_4711)? {
                        0 => return Ok(None),
                        1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
                        5 => (),
                        _ => unreachable!("Impossible to read more than 5 bytes into 5 byte array"),
                    };
                    Ok(Some(Self::Did4711(did_4711)))
                }
                0x8711 => {
                    let mut did_8711 = [0u8; 5];
                    match reader.read(&mut did_8711)? {
                        0 => return Ok(None),
                        1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
                        5 => (),
                        _ => unreachable!("Impossible to read more than 5 bytes into 5 byte array"),
                    };
                    Ok(Some(Self::Did8711(did_8711)))
                }
                _ => Err(Error::IncorrectMessageLengthOrInvalidFormat),
            }
        }

        fn required_size(&self) -> usize {
            2 + match self {
                ProtocolPayload::Did4711(_) => 5,
                ProtocolPayload::Did8711(_) => 5,
            }
        }

        fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
            writer.write_u16::<byteorder::BigEndian>(self.value())?;
            let mut written = 2;

            match self {
                ProtocolPayload::Did4711(data) => {
                    writer.write_all(data)?;
                    written += 5;
                }
                ProtocolPayload::Did8711(data) => {
                    writer.write_all(data)?;
                    written += 5;
                }
            }
            Ok(written)
        }
    }

    impl IterableWireFormat for ProtocolPayload {}

    use super::*;

    #[test]
    fn snapshot_record() {
        let record = DTCSnapshotRecordNumber(0x01);
        let mut writer = Vec::new();
        let written_number = record.to_writer(&mut writer).unwrap();
        assert_eq!(record.required_size(), 1);
        assert_eq!(written_number, 1);
    }

    #[test]
    fn snapshot_list() {
        #[rustfmt::skip]
        let bytes:[u8; 20] = [
            // DTC Number + Status
            0x12, 0x34, 0x56, 0x24, 
            // DTC Snapshot Record Number
            0x02, 
            // Number of DIDs to read (0 indicates an unknown number???)
            0x00, 
            // DID (fake)
            0x47, 0x11, 
            // Snapshot data
            0xA6, 0x66, 0x07, 0x50, 0x20,
            0x87, 0x11,
            0x00, 0x00, 0x00, 0x00, 0x09,
        ];

        let resp = DTCSnapshotRecordList::from_reader(&mut bytes.as_slice())
            .unwrap();

        assert_eq!(resp.mask_record, DTCMaskRecord::from(0x123456));

        resp.snapshot_data
            .iter()
            .for_each(|(record_number, record)| {
                assert_eq!(record_number.0, 0x02);
                // check the data of the payload
                for payload in &record.data {
                    match payload {
                        ProtocolPayload::Did4711(data) => {
                            assert_eq!(data, &[0xA6, 0x66, 0x07, 0x50, 0x20]);
                        }
                        ProtocolPayload::Did8711(data) => {
                            assert_eq!(data, &[0x00, 0x00, 0x00, 0x00, 0x09]);
                        }
                    }
                    let mut writer = Vec::new();
                    let written = payload.to_writer( &mut writer).unwrap();
                    assert_eq!(written, payload.required_size());

                    
                }

            })
    }
}
