//! Routine Control (0x31) Service is used to perform functions on the ECU that are may not be covered by other services.
//!
//! It can also be used to check the ECU’s health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::{Error, Identifier, RoutineControlSubFunction, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Used by a client to execute a defined sequence of events and obtain any relevant results
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<RoutineIdentifier, RoutinePayload> {
    pub sub_function: RoutineControlSubFunction,
    pub routine_id: RoutineIdentifier,
    pub data: Option<RoutinePayload>,
}

impl<RoutineIdentifier: Identifier, RoutinePayload: WireFormat>
    RoutineControlRequest<RoutineIdentifier, RoutinePayload>
{
    pub(crate) fn new(
        sub_function: RoutineControlSubFunction,
        routine_id: RoutineIdentifier,
        data: Option<RoutinePayload>,
    ) -> Self {
        Self {
            sub_function,
            routine_id,
            data,
        }
    }
}

impl<RoutineIdentifier: Identifier, RoutinePayload: WireFormat> WireFormat
    for RoutineControlRequest<RoutineIdentifier, RoutinePayload>
{
    fn decode<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let sub_function = RoutineControlSubFunction::from(reader.read_u8()?);
        let routine_id = RoutineIdentifier::decode(reader)?.unwrap();
        let data = RoutinePayload::decode(reader)?;
        Ok(Some(Self {
            sub_function,
            routine_id,
            data,
        }))
    }

    fn required_size(&self) -> usize {
        3 + match &self.data {
            Some(record) => record.required_size(),
            None => 0,
        }
    }

    fn encode<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.sub_function))?;
        self.routine_id.encode(writer)?;
        if let Some(record) = &self.data {
            record.encode(writer)?;
        }
        Ok(self.required_size())
    }
}

impl<RoutineIdentifier: Identifier, RoutinePayload: WireFormat> SingleValueWireFormat
    for RoutineControlRequest<RoutineIdentifier, RoutinePayload>
{
}

/// `RoutineControlResponse` is a variable length field that can contain the status of the routine
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<RoutineInfoStatusRecord> {
    /// The sub-function echoes the routine control request
    pub routine_control_type: RoutineControlSubFunction,

    /// Should contain the `routine_info` (u8) and the `routine_status_record` (u8 * n) information. n can be 0
    ///
    /// `routine_info`: The routine information that the response is for (vehicle manufacturer specific)
    /// `routine_status_record`: The status of the routine (optional)
    ///
    /// Mandatory for any routine where the `routine_status_record` is defined by ISO/SAE specs, even if it is 0 bytes.
    /// Optional if the routine is defined by a manufacturer.
    pub routine_status_record: RoutineInfoStatusRecord,
}

impl<RoutineStatusRecord: WireFormat> RoutineControlResponse<RoutineStatusRecord> {
    pub(crate) fn new(
        routine_control_type: RoutineControlSubFunction,
        data: RoutineStatusRecord,
    ) -> Self {
        Self {
            routine_control_type,
            routine_status_record: data,
        }
    }

    /// Get the raw data of the status record
    /// # Errors
    /// - if the stream is not in the expected format
    /// - if the stream contains partial data
    pub fn status_record_data(&self) -> Result<Vec<u8>, Error> {
        let mut writer: Vec<u8> = Vec::new();
        self.routine_status_record.encode(&mut writer)?;

        Ok(writer)
    }
}

impl<RoutineStatusRecord: WireFormat> WireFormat for RoutineControlResponse<RoutineStatusRecord> {
    fn decode<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let routine_control_type = RoutineControlSubFunction::from(reader.read_u8()?);
        // Reads the identifier, then can read 0 bytes, 1 byte, or more
        let routine_status_record = RoutineStatusRecord::decode(reader)?.unwrap();
        Ok(Some(Self {
            routine_control_type,
            routine_status_record,
        }))
    }

    /// Can be 3 bytes, or more
    fn required_size(&self) -> usize {
        // control type + (routine identifier + routine info + status record)
        1 + self.routine_status_record.required_size()
    }

    fn encode<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.routine_control_type.into())?;
        self.routine_status_record.encode(writer)?;
        Ok(self.required_size())
    }
}

impl<RoutineStatusRecord: WireFormat> SingleValueWireFormat
    for RoutineControlResponse<RoutineStatusRecord>
{
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::Identifier;

    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    #[derive(Clone, Copy, Debug, Eq, Identifier, PartialEq)]
    struct TestIdentifier(pub u16);

    impl From<u16> for TestIdentifier {
        fn from(value: u16) -> Self {
            TestIdentifier(value)
        }
    }

    impl From<TestIdentifier> for u16 {
        fn from(val: TestIdentifier) -> Self {
            val.0
        }
    }

    type RoutineControlRequestType = RoutineControlRequest<TestIdentifier, Vec<u8>>;

    #[test]
    fn simple_request() {
        // Fake data: StartRoutine, RoutineID of 0x8606 for "Start O2 Sensor Heater Test" or something
        let bytes: [u8; 6] = [0x01, 0x00, 0x01, 0x02, 0x03, 0x04];
        let req: RoutineControlRequestType =
            RoutineControlRequest::decode_single_value(&mut bytes.as_slice()).unwrap();

        assert_eq!(u8::from(req.sub_function), 0x01);
        assert_eq!(req.routine_id, TestIdentifier::from(0x0001));
        let data = req.data.clone().unwrap();
        assert_eq!(data, vec![0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = req.encode(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.required_size());

        let new_req: RoutineControlRequestType = RoutineControlRequest::new(
            RoutineControlSubFunction::StopRoutine,
            TestIdentifier::from(0x0002),
            Some(vec![]),
        );

        assert_eq!(new_req.sub_function, RoutineControlSubFunction::StopRoutine);
        assert_eq!(new_req.routine_id, TestIdentifier::from(0x0002));
    }

    #[test]
    fn simple_response() {
        let bytes: [u8; 6] = [0x01, 0x00, 0x01, 0x02, 0x03, 0x04];
        let resp: RoutineControlResponse<Vec<u8>> =
            RoutineControlResponse::decode_single_value(&mut bytes.as_slice()).unwrap();

        assert_eq!(
            resp.routine_control_type,
            RoutineControlSubFunction::StartRoutine
        );
        // Vec<u8> as payload just reads until the end, including the identifier
        assert_eq!(
            resp.routine_status_record,
            vec![0x00, 0x01, 0x02, 0x03, 0x04]
        );

        let mut buf = Vec::new();
        let written = resp.encode(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, resp.required_size());

        let new_resp: RoutineControlResponse<Vec<u8>> =
            RoutineControlResponse::new(RoutineControlSubFunction::StopRoutine, buf);

        assert_eq!(
            new_resp.routine_control_type,
            RoutineControlSubFunction::StopRoutine
        );
    }
}
