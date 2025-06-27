//! Routine Control (0x31) Service is used to perform functions on the ECU that are may not be covered by other services.
//!
//! It can also be used to check the ECU’s health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::{Error, Identifier, RoutineControlSubFunction, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Used by a client to execute a defined sequence of events and obtain any relevant results
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let sub_function = RoutineControlSubFunction::from(reader.read_u8()?);
        let routine_id = RoutineIdentifier::option_from_reader(reader)?.unwrap();
        let data = RoutinePayload::option_from_reader(reader)?;
        Ok(Some(Self {
            sub_function,
            routine_id,
            data,
        }))
    }

    fn required_size(&self) -> usize {
        3 + match &self.data {
            Some(ref record) => record.required_size(),
            None => 0,
        }
    }

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.sub_function))?;
        self.routine_id.to_writer(writer)?;
        if let Some(record) = &self.data {
            record.to_writer(writer)?;
        }
        Ok(self.required_size())
    }
}

impl<RoutineIdentifier: Identifier, RoutinePayload: WireFormat> SingleValueWireFormat
    for RoutineControlRequest<RoutineIdentifier, RoutinePayload>
{
}

/// RoutineControlResponse is a variable length field that can contain the status of the routine
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<RoutineIdentifier, RoutineInfoStatusRecord> {
    /// The sub-function echoes the routine control request
    pub routine_control_type: RoutineControlSubFunction,

    pub routine_id: RoutineIdentifier,

    /// Should contain the routine_info (u8) and the routine_status_record (u8 * n) information. n can be 0
    ///
    /// routine_info: The routine information that the response is for (vehicle manufacturer specific)
    /// routine_status_record: The status of the routine (optional)
    ///
    /// Mandatory for any routine where the routine_status_record is defined by ISO/SAE specs, even if it is 0 bytes.
    /// Optional if the routine is defined by a manufacturer.
    pub routine_status_record: RoutineInfoStatusRecord,
}

impl<RoutineIdentifier: Identifier, RoutineStatusRecord: WireFormat>
    RoutineControlResponse<RoutineIdentifier, RoutineStatusRecord>
{
    pub(crate) fn new(
        routine_control_type: RoutineControlSubFunction,
        routine_id: RoutineIdentifier,
        data: RoutineStatusRecord,
    ) -> Self {
        Self {
            routine_control_type,
            routine_id,
            routine_status_record: data,
        }
    }

    pub fn identifier(&self) -> RoutineIdentifier {
        self.routine_id
    }

    /// Get the raw data of the status record
    pub fn status_record_data(&self) -> Result<Vec<u8>, Error> {
        let mut writer: Vec<u8> = Vec::new();
        self.routine_status_record.to_writer(&mut writer)?;

        Ok(writer)
    }
}

impl<RoutineIdentifier: Identifier, RoutineStatusRecord: WireFormat> WireFormat
    for RoutineControlResponse<RoutineIdentifier, RoutineStatusRecord>
{
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let routine_control_type = RoutineControlSubFunction::from(reader.read_u8()?);
        let routine_id = RoutineIdentifier::option_from_reader(reader)?.unwrap();
        // can read 0 bytes, 1 byte, or more
        let routine_status_record = RoutineStatusRecord::option_from_reader(reader)?.unwrap();
        Ok(Some(Self {
            routine_control_type,
            routine_id,
            routine_status_record,
        }))
    }

    /// Can be 3 bytes, or more
    fn required_size(&self) -> usize {
        // control type + (routine identifier + routine info + status record)
        1 + self.routine_id.required_size() + self.routine_status_record.required_size()
    }

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.routine_control_type.into())?;
        self.routine_id.to_writer(writer)?;
        self.routine_status_record.to_writer(writer)?;
        Ok(self.required_size())
    }
}

impl<RoutineIdentifier: Identifier, RoutineStatusRecord: WireFormat> SingleValueWireFormat
    for RoutineControlResponse<RoutineIdentifier, RoutineStatusRecord>
{
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::Identifier;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct MockU16(pub u16);

    impl Identifier for MockU16 {}

    impl From<u16> for MockU16 {
        fn from(value: u16) -> Self {
            MockU16(value)
        }
    }

    impl From<MockU16> for u16 {
        fn from(val: MockU16) -> Self {
            val.0
        }
    }

    type RoutineControlRequestType = RoutineControlRequest<MockU16, Vec<u8>>;

    #[test]
    fn simple_request() {
        // Fake data: StartRoutine, RoutineID of 0x8606 for "Start O2 Sensor Heater Test" or something
        let bytes: [u8; 6] = [0x01, 0x00, 0x01, 0x02, 0x03, 0x04];
        let req: RoutineControlRequestType =
            RoutineControlRequest::from_reader(&mut bytes.as_slice()).unwrap();

        assert_eq!(u8::from(req.sub_function), 0x01);
        assert_eq!(req.routine_id, MockU16::from(0x01));
        let data = req.data.clone().unwrap();
        assert_eq!(data, vec![0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = req.to_writer(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.required_size());

        let new_req: RoutineControlRequestType = RoutineControlRequest::new(
            RoutineControlSubFunction::StopRoutine,
            MockU16::from(0x02),
            Some(vec![]),
        );

        assert_eq!(new_req.sub_function, RoutineControlSubFunction::StopRoutine);
        assert_eq!(new_req.routine_id, MockU16::from(0x02));
    }

    #[test]
    fn simple_response() {
        let bytes: [u8; 6] = [0x01, 0x00, 0x01, 0x02, 0x03, 0x04];
        let resp: RoutineControlResponse<MockU16, Vec<u8>> =
            RoutineControlResponse::from_reader(&mut bytes.as_slice()).unwrap();

        assert_eq!(u8::from(resp.routine_control_type), 0x01);
        assert_eq!(resp.routine_status_record, vec![0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = resp.to_writer(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, resp.required_size());

        let new_resp: RoutineControlResponse<MockU16, Vec<u8>> = RoutineControlResponse::new(
            RoutineControlSubFunction::StopRoutine,
            MockU16::from(0xFF00),
            buf,
        );

        assert_eq!(
            new_resp.routine_control_type,
            RoutineControlSubFunction::StopRoutine
        );
    }
}
