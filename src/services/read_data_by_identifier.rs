use crate::{
    DataIdentifier, Error, IterableWireFormat, NegativeResponseCode, SingleValueWireFormat,
    WireFormat,
};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

#[non_exhaustive]
pub struct ReadDataByIdentifierRequest<I: SingleValueWireFormat<Error>> {
    pub dids: Vec<DataIdentifier<I>>,
}

impl<I> ReadDataByIdentifierRequest<I>
where
    I: SingleValueWireFormat<Error>,
{
    pub(crate) fn new(dids: Vec<DataIdentifier<I>>) -> Self {
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl WireFormat for ReadDataByIdentifierRequest {
    /// Create a TesterPresentResponse from a sequence of bytes
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut dids = Vec::new();
        for identifier in DataIdentifier::from_reader_iterable(reader) {
            match identifier {
                Ok(id) => {
                    dids.push(id);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(Some(Self { dids }))
    }

    /// Write the response as a sequence of bytes to a buffer
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut count = 0;
        for did in &self.dids {
            did.to_writer(writer)?;
            count += 2;
        }
        Ok(count)
    }
}

impl SingleValueWireFormat<Error> for ReadDataByIdentifierRequest {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]

pub enum TestDID {
    TestData,
}

impl TestDID {
    // Function to map byte values to enum variants
    fn from_byte(byte: u16) -> Option<Self> {
        match byte {
            0x42 => Some(TestDID::TestData),
            _ => None, // Handle out-of-range byte values if needed
        }
    }
}

pub struct ReadDataByIdentifierResponse {
    pub dids: Vec<u16>,
    pub did_records: Vec<Vec<u8>>,
}

impl ReadDataByIdentifierResponse {
    pub(crate) fn new(dids: Vec<u16>, did_records: Vec<Vec<u8>>) -> Self {
        Self { dids, did_records }
    }
}

impl WireFormat<Error> for ReadDataByIdentifierResponse {
    /// Create a TesterPresentResponse from a sequence of bytes
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut dids = Vec::new();
        let mut did_records = Vec::new();
        let mut first_pass = true;
        let mut have_data = false;

        loop {
            let mut buf = [0u8; 2];
            let bytes_read = reader.read_exact(&mut buf);

            // Check if there is data left in the reader
            match bytes_read {
                Ok(()) => {
                    let did = u16::from_le_bytes(buf);
                    // TODO: Check if valid did
                    dids.push(did);
                }
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    return Err(Error::InsufficientData(4));
                }
                Err(e) => {
                    return Err(Error::IoError(e));
                }
            }

            // TODO: Lookup the correct number of bytes for this did
            let mut record = vec![0u8; 4];
            let bytes_read = reader.read_exact(&mut record);

            match bytes_read {
                Ok(()) => {
                    did_records.push(record);
                }
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    // If we encounter EOF after reading a did, that's an error
                    // TODO: Report correct error count
                    return Err(Error::InsufficientData(4));
                }
                Err(e) => {
                    return Err(Error::IoError(e));
                }
            }

            // Check if there are more bytes in the reader
            if reader.bytes().next().is_none() {
                break; // End of data
            }
        }

        if dids.is_empty() && did_records.is_empty() {
            Err(Error::InsufficientData(6)) // No data at all
        } else {
            Ok(Some(ReadDataByIdentifierResponse { dids, did_records }))
        }
    }
}

/// Write the response as a sequence of bytes to a buffer
fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
    if self.dids.is_empty() || self.did_records.is_empty() {
        return Err(Error::NoData); // No data at all
    }

    if self.dids.len() != self.did_records.len() {
        return Err(Error::InconsistentData); // Mismatch between dids and did_records
    }

    let mut total_written = 0;

    for (did, record) in self.dids.iter().zip(&self.did_records) {
        // Write the u16 (did)
        let did_bytes = did.to_le_bytes();
        writer.write_all(&did_bytes)?;

        // Write the 4 bytes of data
        writer.write_all(record)?;

        total_written += did_bytes.len() + record.len();
    }

    Ok(total_written)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

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
        let test_data_sets = vec![
            vec![],
            vec![1u8],
            vec![1u8, 2u8],
            vec![1u8, 2u8, 3u8],
            vec![1u8, 2u8, 3u8, 4u8],
            vec![1u8, 2u8, 3u8, 4u8, 5u8],
            vec![1u8, 2u8, 3u8, 4u8, 5u8, 6u8],
            (0u8..=255).collect::<Vec<u8>>(),
        ];

        for bytes in test_data_sets.iter() {
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
    fn write_did_request_bytes() {
        let requests = vec![
            ReadDataByIdentifierRequest::new(vec![]),
            ReadDataByIdentifierRequest::new(vec![0u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16, 5u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16, 5u16, 6u16]),
            ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16, 5u16, 6u16, 7u16]),
            ReadDataByIdentifierRequest::new((0..u16::MAX - 1).collect::<Vec<u16>>()),
            ReadDataByIdentifierRequest::new((0..u16::MAX).collect::<Vec<u16>>()),
        ];

        for request in requests {
            let mut buffer = Vec::new();
            let result = request.to_writer(&mut buffer);

            match result {
                Ok(bytes_read) => {
                    // 1 did is 2 bytes
                    let expected_byte_count = request.dids.len() * 2;
                    assert_eq!(bytes_read, expected_byte_count);
                }
                _ => {
                    assert!(matches!(result.unwrap_err(), Error::InsufficientData(_)));
                }
            }
        }
    }
}
