use crate::{Error, IterableWireFormat, NegativeResponseCode, SingleValueWireFormat, WireFormat};
use serde::{Deserialize, Serialize};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ReadDataByIdentifierRequest<Identifier> {
    pub dids: Vec<Identifier>,
}

impl<Identifier: IterableWireFormat> ReadDataByIdentifierRequest<Identifier> {
    pub(crate) fn new(dids: Vec<Identifier>) -> Self {
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<Identifier: IterableWireFormat> WireFormat for ReadDataByIdentifierRequest<Identifier> {
    /// Create a TesterPresentResponse from a sequence of bytes
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let mut dids = Vec::new();
        for identifier in Identifier::from_reader_iterable(reader) {
            match identifier {
                Ok(id) => {
                    dids.push(id);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        if dids.is_empty() {
            // TODO: Add more specific error here
            Err(Error::InsufficientData(0)) // No data at all
        } else {
            Ok(Some(Self { dids }))
        }
    }

    /// Write the response as a sequence of bytes to a buffer
    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut count = 0;
        for did in &self.dids {
            did.to_writer(writer)?;
            count += 2;
        }
        Ok(count)
    }
}

impl<Identifier: IterableWireFormat> SingleValueWireFormat
    for ReadDataByIdentifierRequest<Identifier>
{
}

pub struct ReadDataByIdentifierResponse<UserPayload> {
    pub data: Vec<UserPayload>,
}

impl<UserPayload> ReadDataByIdentifierResponse<UserPayload> {
    pub(crate) fn new(data: Vec<UserPayload>) -> Self {
        Self { data }
    }
}

impl<UserPayload: IterableWireFormat> WireFormat for ReadDataByIdentifierResponse<UserPayload> {
    /// Create a TesterPresentResponse from a sequence of bytes
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let mut data = Vec::new();
        for payload in UserPayload::from_reader_iterable(reader) {
            match payload {
                Ok(p) => {
                    data.push(p);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        if data.is_empty() {
            // TODO: More descriptive error type
            Err(Error::InsufficientData(0)) // No data at all
        } else {
            Ok(Some(Self { data }))
        }
    }

    /// Write the response as a sequence of bytes to a buffer
    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut total_written = 0;
        for payload in &self.data {
            total_written += payload.to_writer(writer)?;
        }
        Ok(total_written)
    }
}

impl<UserPayload: IterableWireFormat> SingleValueWireFormat
    for ReadDataByIdentifierResponse<UserPayload>
{
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
