use crate::{Error, NegativeResponseCode, SingleValueWireFormat, WireFormat};
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use num::Integer;
use serde::{Deserialize, Serialize};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReadDataByIdentifierRequest {
    pub dids: Vec<u16>,
}

impl ReadDataByIdentifierRequest {
    pub(crate) fn new(dids: Vec<u16>) -> Self {
        Self { dids }
    }
}

impl WireFormat for ReadDataByIdentifierRequest {
    /// Create a TesterPresentResponse from a sequence of bytes
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut input_data: Vec<u8> = Vec::new();
        reader.read_to_end(&mut input_data)?;

        let data_length = input_data.len();

        if data_length == 0 {
            return Err(Error::InsufficientData(2));
        }

        // Since dids are u16 (two bytes), an odd number of bytes implies a partial did was received which is an error
        if data_length.is_odd() {
            return Err(Error::InsufficientData(data_length + 1));
        }

        let mut dids = Vec::new();
        for chunk in input_data.chunks_exact(2) {
            let value = BigEndian::read_u16(chunk);
            dids.push(value);
        }

        Ok(Some(Self { dids }))
    }

    /// Write the response as a sequence of bytes to a buffer
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut count = 0;
        for did in self.dids.iter() {
            writer.write_u16::<BigEndian>(*did)?;
            count += 2;
        }

        Ok(count)
    }
}

impl SingleValueWireFormat for ReadDataByIdentifierRequest {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TestDataRecord {
    pub data_record: [u16; 4],
}

// pub struct ReadDataByIdentifierResponse {
//     pub dids: Vec<u16>,
//     pub did_record: Vec<TestDataRecord>,
// }

#[cfg(test)]
mod test {
    // use num::iter;

    use super::*;
    use std::io::Cursor;

    // fn make_did_bytes(lower: u8, upper: u8) -> Vec<u8> {
    //     (lower..=upper).collect::<Vec<u8>>()
    // }

    fn bytes_to_dids(bytes: &Vec<u8>) -> Option<Vec<u16>> {
        if bytes.len().is_odd() {
            return None;
        }

        let mut dids = vec![];

        for chunk in bytes.chunks_exact(2) {
            let value = BigEndian::read_u16(chunk);
            dids.push(value);
        }

        Some(dids)
    }

    #[test]
    fn read_did_request_bytes() {
        // struct TestData {
        //     pub test_id: u8,
        //     pub dids_bytes: Vec<u8>,
        // }

        // let src_test_bytes = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8];
        // let mut test_sets: Vec<Vec<u8>> = vec![vec![]];

        // for (i, _) in src_test_bytes.iter().enumerate() {
        //     test_sets.push(src_test_bytes[0..i].to_vec());
        // }

        let test_sets = vec![
            vec![],
            vec![1u8],
            vec![1u8, 2u8],
            vec![1u8, 2u8, 3u8],
            vec![1u8, 2u8, 3u8, 4u8],
            vec![1u8, 2u8, 3u8, 4u8, 5u8],
            vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8],
            (0u8..=255).collect::<Vec<u8>>(),
        ];

        for bytes in test_sets.iter() {
            let len = bytes.len();
            let mut byte_access = Cursor::new(bytes);
            let read_result = ReadDataByIdentifierRequest::option_from_reader(&mut byte_access);

            if len == 0 || len.is_odd() {
                assert!(matches!(read_result, Err(Error::InsufficientData(_))));
            } else {
                let dids = bytes_to_dids(bytes).unwrap();
                assert_eq!(
                    read_result.unwrap().unwrap(),
                    ReadDataByIdentifierRequest::new(dids)
                );
            }
        }
    }

    #[test]
    fn write_did_request_bytes() {}
}
