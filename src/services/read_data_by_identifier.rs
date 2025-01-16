use crate::{
    Error, IterableWireFormat, NegativeResponseCode, ProtocolIdentifier, SingleValueWireFormat,
    UDSIdentifier, WireFormat,
};

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

    // fn bytes_to_dids(bytes: &Vec<u8>) -> Option<Vec<u16>> {
    //     if bytes.len().is_odd() {
    //         return None;
    //     }

    //     let mut dids = vec![];

    //     for chunk in bytes.chunks_exact(2) {
    //         let value = BigEndian::read_u16(chunk);
    //         dids.push(value);
    //     }

    //     Some(dids)
    // }

    fn to_bytes(ids: &[ProtocolIdentifier]) -> Vec<u8> {
        ids.iter()
            .map(|id: &ProtocolIdentifier| {
                let mut buffer = Vec::new();
                id.to_writer(&mut buffer).unwrap();
                buffer
            })
            .flatten()
            .collect()
    }

    #[test]
    fn read_did_request_bytes() {
        let test_ids: Vec<ProtocolIdentifier> = vec![
            ProtocolIdentifier::new(UDSIdentifier::BootSoftwareIdentification),
            ProtocolIdentifier::new(UDSIdentifier::ApplicationSoftware),
            ProtocolIdentifier::new(UDSIdentifier::ApplicationDataIdentification),
            ProtocolIdentifier::new(UDSIdentifier::BootSoftwareFingerprint),
            ProtocolIdentifier::new(UDSIdentifier::ApplicationSoftwareFingerprint),
            ProtocolIdentifier::new(UDSIdentifier::ApplicationDataFingerprint),
            ProtocolIdentifier::new(UDSIdentifier::ActiveDiagnosticSession),
            ProtocolIdentifier::new(UDSIdentifier::VehicleManufacturerSparePartNumber),
            ProtocolIdentifier::new(UDSIdentifier::VehicleManufacturerECUSoftwareNumber),
            ProtocolIdentifier::new(UDSIdentifier::VehicleManufacturerECUSoftwareVersionNumber),
        ];

        // Create test data sets using the `serialize_ids_to_bytes` function
        let test_data_sets: Vec<Vec<u8>> = vec![
            vec![],                    // No ids
            vec![0x00],                // Invalid byte length
            vec![0x00, 0x01],          // Invalid id
            to_bytes(&test_ids[0..1]), // First id
            to_bytes(&test_ids[0..2]), // First two ids
            to_bytes(&test_ids[0..3]),
            to_bytes(&test_ids[0..4]),
            to_bytes(&test_ids), // All ids
            to_bytes(
                &test_ids
                    .iter()
                    .cycle()
                    .take(100)
                    .cloned()
                    .collect::<Vec<_>>(),
            ), // Repeated ids, 100 items
        ];

        for (test_index, id_bytes) in test_data_sets.iter().enumerate() {
            let mut byte_access = Cursor::new(id_bytes);
            let read_result = ReadDataByIdentifierRequest::<ProtocolIdentifier>::option_from_reader(
                &mut byte_access,
            );

            match read_result {
                Ok(Some(response)) => {
                    let mut translated_bytes = Vec::new();
                    response.to_writer(&mut translated_bytes).unwrap();

                    // Now, compare `dids_bytes` with `id_bytes` (excluding leading/trailing data if necessary)
                    assert_eq!(
                        translated_bytes, *id_bytes,
                        "Ok: Failed on index {}",
                        test_index
                    );
                }
                Ok(None) => {
                    // No data read
                    assert!(id_bytes.is_empty(), "None: Failed on index {}", test_index);
                }
                Err(e) => {
                    if id_bytes.is_empty() {
                        assert!(
                            matches!(e, Error::InsufficientData(_)),
                            "InsufficientData: Failed on index {}",
                            test_index
                        );
                        break;
                    } else {
                        assert!(
                            matches!(e, Error::IncorrectMessageLengthOrInvalidFormat),
                            "IncorrectMessageLengthOrInvalidFormat: Failed on index {}",
                            test_index
                        );
                    }
                }
            }
        }
    }

    // #[test]
    // fn write_did_request_bytes() {
    //     let requests = vec![
    //         ReadDataByIdentifierRequest::new(vec![]),
    //         ReadDataByIdentifierRequest::new(vec![0u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16, 5u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16, 5u16, 6u16]),
    //         ReadDataByIdentifierRequest::new(vec![0u16, 1u16, 2u16, 3u16, 4u16, 5u16, 6u16, 7u16]),
    //         ReadDataByIdentifierRequest::new((0..u16::MAX - 1).collect::<Vec<u16>>()),
    //         ReadDataByIdentifierRequest::new((0..u16::MAX).collect::<Vec<u16>>()),
    //     ];

    //     for request in requests {
    //         let mut buffer = Vec::new();
    //         let result = request.to_writer(&mut buffer);

    //         match result {
    //             Ok(bytes_read) => {
    //                 // 1 did is 2 bytes
    //                 let expected_byte_count = request.dids.len() * 2;
    //                 assert_eq!(bytes_read, expected_byte_count);
    //             }
    //             _ => {
    //                 assert!(matches!(result.unwrap_err(), Error::InsufficientData(_)));
    //             }
    //         }
    //     }
    // }
}
