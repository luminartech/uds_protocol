use crate::{Error, IterableWireFormat, NegativeResponseCode, SingleValueWireFormat, WireFormat};

use serde::{Deserialize, Serialize};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// See ISO-14229-1:2020, Table 11.2.1 for format information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ReadDataByIdentifierRequest<Identifier> {
    pub dids: Vec<Identifier>,
}

impl<Identifier: IterableWireFormat> ReadDataByIdentifierRequest<Identifier> {
    /// Create a new request from a sequence of data identifiers
    pub(crate) fn new(dids: Vec<Identifier>) -> Self {
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl<Identifier: IterableWireFormat> WireFormat for ReadDataByIdentifierRequest<Identifier> {
    /// Create a request from a sequence of bytes
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
            Err(Error::NoDataAvailable)
        } else {
            Ok(Some(ReadDataByIdentifierRequest::new(dids)))
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

/// See ISO-14229-1:2020, Table 11.2.3 for format information
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReadDataByIdentifierResponse<UserPayload> {
    pub data: Vec<UserPayload>,
}

impl<UserPayload> ReadDataByIdentifierResponse<UserPayload> {
    pub(crate) fn new(data: Vec<UserPayload>) -> Self {
        Self { data }
    }
}

impl<UserPayload: IterableWireFormat> WireFormat for ReadDataByIdentifierResponse<UserPayload> {
    /// Create a response from a sequence of bytes
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
            Err(Error::NoDataAvailable)
        } else {
            Ok(Some(ReadDataByIdentifierResponse::new(data)))
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
    use crate::{ProtocolIdentifier, UDSIdentifier};
    use std::io::Cursor;

    mod request {
        use super::*;

        struct ReadRequestTestData {
            pub test_name: String,
            pub dids_bytes: Vec<u8>,
        }

        impl ReadRequestTestData {
            // Creates a Test Read Request from a list of data identifiers
            fn from_ids(test_name: &str, dids: &[ProtocolIdentifier]) -> Self {
                let dids_bytes = to_bytes(dids);
                Self {
                    test_name: test_name.to_string(),
                    dids_bytes,
                }
            }

            // Create a Test Read Request from a list of bytes.
            // Note: These bytes may not properly translate to a list of data identifiers
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
                .flat_map(|id: &ProtocolIdentifier| {
                    let mut buffer = Vec::new();
                    id.to_writer(&mut buffer).unwrap();
                    buffer
                })
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
                let read_result =
                    ReadDataByIdentifierRequest::<ProtocolIdentifier>::option_from_reader(
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
                                matches!(e, Error::NoDataAvailable),
                                "NoDataAvailable: Failed {}",
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
                        assert!(
                            matches!(e, Error::InsufficientData(_)),
                            "InsufficientData: Failed {}",
                            test_data.test_name
                        );
                    }
                }
            }
        }
    }

    mod response {
        use super::*;

        #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
        pub struct BazData {
            pub data: [u8; 16],
            pub data2: u64,
            pub data3: u16,
        }

        // The UDSIdentifiers are vender defined and don't have interesting payloads, so we define our own types for
        #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
        pub enum TestPayload {
            #[serde(with = "serde_bytes")]
            MeaningOfLife([u8; 42]),
            Foo(u32),
            Bar,
            Baz(BazData),
            UDSIdentifier(UDSIdentifier),
        }

        impl TestPayload {
            fn new<R: std::io::Read>(did: u16, reader: &mut R) -> Result<Self, Error> {
                match did {
                    0xFF00 => {
                        let mut data = [0u8; 42];
                        reader.read_exact(&mut data)?;
                        Ok(TestPayload::MeaningOfLife(data))
                    }
                    0xFF01 => {
                        let mut data = [0u8; 4];
                        reader.read_exact(&mut data)?;
                        let value = u32::from_be_bytes(data);
                        Ok(TestPayload::Foo(value))
                    }
                    0xFF02 => Ok(TestPayload::Bar),
                    0xFF03 => {
                        let data = BazData::option_from_reader(reader)?.unwrap();
                        Ok(TestPayload::Baz(data))
                    }
                    _ => {
                        let identifier = UDSIdentifier::try_from(did)?;
                        Ok(TestPayload::UDSIdentifier(identifier))
                    }
                }
            }
        }

        impl From<TestPayload> for u16 {
            fn from(value: TestPayload) -> Self {
                match value {
                    TestPayload::MeaningOfLife(_) => 0xFF00,
                    TestPayload::Foo(_) => 0xFF01,
                    TestPayload::Bar => 0xFF02,
                    TestPayload::Baz(_) => 0xFF03,
                    TestPayload::UDSIdentifier(uds_id) => u16::from(uds_id),
                }
            }
        }

        impl IterableWireFormat for TestPayload {}

        impl WireFormat for TestPayload {
            fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
                let mut identifier_data: [u8; 2] = [0; 2];
                match reader.read(&mut identifier_data)? {
                    0 => return Ok(None),
                    1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
                    2 => (),
                    _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
                };
                let did = u16::from_be_bytes(identifier_data);
                Ok(Some(TestPayload::new(did, reader)?))
            }

            fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                let id_bytes = u16::from(self.clone()).to_be_bytes();
                let did_len = writer.write(&id_bytes)?;
                match self {
                    TestPayload::MeaningOfLife(data) => {
                        writer.write_all(data)?;
                        Ok(did_len + data.len())
                    }
                    TestPayload::Foo(value) => {
                        let bytes = value.to_be_bytes();
                        writer.write_all(&bytes)?;
                        Ok(did_len + bytes.len())
                    }
                    TestPayload::Bar => Ok(did_len),
                    TestPayload::Baz(data) => data.to_writer(writer),
                    TestPayload::UDSIdentifier(_) => Ok(did_len),
                }
            }
        }

        impl WireFormat for BazData {
            fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
                let mut data = [0u8; 16];
                reader.read_exact(&mut data)?;

                let mut data2_bytes = [0u8; 8];
                reader.read_exact(&mut data2_bytes)?;
                let data2 = u64::from_be_bytes(data2_bytes);

                let mut data3_bytes = [0u8; 2];
                reader.read_exact(&mut data3_bytes)?;
                let data3 = u16::from_be_bytes(data3_bytes);

                Ok(Some(BazData { data, data2, data3 }))
            }

            fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.write_all(&self.data)?;
                let mut count = 16;
                count += writer.write(&self.data2.to_be_bytes())?;
                count += writer.write(&self.data3.to_be_bytes())?;
                // 2 for the initial did bytes
                Ok(2 + count)
            }
        }

        fn get_test_response_data() -> Vec<TestPayload> {
            vec![
                TestPayload::MeaningOfLife([0; 42]),
                TestPayload::Foo(42),
                TestPayload::Bar,
                TestPayload::Baz(BazData {
                    data: [5; 16],
                    data2: 1234567890,
                    data3: 54321,
                }),
                TestPayload::UDSIdentifier(UDSIdentifier::BootSoftwareIdentification),
            ]
        }

        #[test]
        fn read_did_response_bytes() {
            let test_data = get_test_response_data();

            let response = ReadDataByIdentifierResponse::new(test_data);
            let mut buffer = Vec::new();
            response.to_writer(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            let read_response: ReadDataByIdentifierResponse<TestPayload> =
                ReadDataByIdentifierResponse::<TestPayload>::option_from_reader(&mut cursor)
                    .unwrap()
                    .unwrap();

            assert_eq!(response, read_response);
        }

        #[test]
        fn write_did_response_bytes() {
            let test_data = get_test_response_data();

            let response = ReadDataByIdentifierResponse::new(test_data.clone());
            let mut buffer = Vec::new();
            let bytes_written = response.to_writer(&mut buffer).unwrap();

            let expected_bytes: Vec<u8> = test_data
                .iter()
                .flat_map(|payload| {
                    let mut buf = Vec::new();
                    payload.to_writer(&mut buf).unwrap();
                    buf
                })
                .collect();

            assert_eq!(buffer, expected_bytes);
            assert_eq!(bytes_written, expected_bytes.len());
        }
    }
}
