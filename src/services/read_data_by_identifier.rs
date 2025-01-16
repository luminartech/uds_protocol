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

    struct ReadRequestTestData {
        pub test_name: String,
        pub dids_bytes: Vec<u8>,
    }

    // Holds a byte array of data that will be transformed into a list of dids.
    impl ReadRequestTestData {
        fn from_ids(test_name: &str, dids: &[ProtocolIdentifier]) -> Self {
            let dids_bytes = to_bytes(&dids);
            Self {
                test_name: test_name.to_string(),
                dids_bytes,
            }
        }

        fn from_bytes(test_name: &str, dids_bytes: Vec<u8>) -> Self {
            Self {
                test_name: test_name.to_string(),
                dids_bytes,
            }
        }
    }

    // Holds a list of dids that will be transformed into a byte sequence
    struct WriteRequestTestData {
        pub test_name: String,
        pub dids: Vec<ProtocolIdentifier>,
    }

    impl WriteRequestTestData {
        fn from_ids(test_name: &str, dids: &[ProtocolIdentifier]) -> Self {
            Self {
                test_name: test_name.to_string(),
                dids: dids.to_vec(),
            }
        }
    }

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

    fn get_test_ids() -> Vec<ProtocolIdentifier> {
        vec![
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
        ]
    }

    #[test]
    fn read_did_request_bytes() {
        let test_ids = get_test_ids();

        let test_data_sets: Vec<ReadRequestTestData> = vec![
            ReadRequestTestData::from_bytes("No ids", vec![]),
            ReadRequestTestData::from_bytes("Invalid byte length", vec![0x00]),
            ReadRequestTestData::from_bytes("Invalid id", vec![0x00, 0x01]),
            ReadRequestTestData::from_ids("1 id", &test_ids[0..1]),
            ReadRequestTestData::from_ids("2 ids", &test_ids[0..2]),
            ReadRequestTestData::from_ids("3 ids", &test_ids[0..3]),
            ReadRequestTestData::from_ids("4 ids", &test_ids[0..4]),
            ReadRequestTestData::from_ids("All ids", &test_ids),
            ReadRequestTestData::from_ids(
                "Repeated ids",
                &test_ids
                    .to_vec()
                    .iter()
                    .cycle()
                    .take(100)
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        ];

        for test_data in test_data_sets.iter() {
            let mut byte_access = Cursor::new(test_data.dids_bytes.to_vec());
            let read_result = ReadDataByIdentifierRequest::<ProtocolIdentifier>::option_from_reader(
                &mut byte_access,
            );

            match read_result {
                Ok(Some(response)) => {
                    let mut translated_bytes = Vec::new();
                    response.to_writer(&mut translated_bytes).unwrap();
                    assert_eq!(
                        translated_bytes, *test_data.dids_bytes,
                        "Some: Failed: {}",
                        test_data.test_name
                    );
                }
                Ok(None) => {
                    // No data read
                    assert!(
                        test_data.dids_bytes.is_empty(),
                        "None: Failed {}",
                        test_data.test_name
                    );
                }
                Err(e) => {
                    if test_data.dids_bytes.is_empty() {
                        assert!(
                            matches!(e, Error::InsufficientData(_)),
                            "InsufficientData: Failed {}",
                            test_data.test_name
                        );
                    } else {
                        assert!(
                            matches!(e, Error::IncorrectMessageLengthOrInvalidFormat),
                            "IncorrectMessageLengthOrInvalidFormat: Failed {}",
                            test_data.test_name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn write_did_request_bytes() {
        let test_ids = get_test_ids();

        let test_data_sets: Vec<WriteRequestTestData> = vec![
            WriteRequestTestData::from_ids("No ids", &Vec::new()),
            WriteRequestTestData::from_ids("1 id", &test_ids[0..1]),
            WriteRequestTestData::from_ids("2 ids", &test_ids[0..2]),
            WriteRequestTestData::from_ids("3 ids", &test_ids[0..3]),
            WriteRequestTestData::from_ids("4 ids", &test_ids[0..4]),
            WriteRequestTestData::from_ids("All ids", &test_ids),
            WriteRequestTestData::from_ids(
                "Repeated ids",
                &test_ids
                    .to_vec()
                    .iter()
                    .cycle()
                    .take(100)
                    .cloned()
                    .collect::<Vec<_>>(),
            ),
        ];

        for test_data in &test_data_sets {
            let request = ReadDataByIdentifierRequest::new(test_data.dids.to_vec());
            let mut buffer = Vec::new();
            let write_result = request.to_writer(&mut buffer);

            match write_result {
                Ok(bytes_read) => {
                    // 1 did is 2 bytes
                    let expected_byte_count = request.dids.len() * 2;
                    assert_eq!(bytes_read, expected_byte_count);
                }
                Err(e) => {
                    // if test_data.dids.is_empty() {
                    assert!(
                        matches!(e, Error::InsufficientData(_)),
                        "InsufficientData: Failed {}",
                        test_data.test_name
                    );
                    // }
                    // else {
                    //     assert!(
                    //         matches!(e, Error::IncorrectMessageLengthOrInvalidFormat),
                    //         "IncorrectMessageLengthOrInvalidFormat: Failed {}",
                    //         test_data.test_name
                    //     );
                    // }
                }
            }
        }
    }
}
