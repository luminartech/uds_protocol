//! Routine Control (0x31) Service is used to perform functions on the ECU that are may not be covered by other services.
//!
//! It can also be used to check the ECU’s health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::{
    Error, IterableWireFormat, RoutineControlSubFunction, SingleValueWireFormat, WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Used by a client to execute a defined sequence of events and obtain any relevant results
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RoutineControlRequest<RoutineIdentifier> {
    pub sub_function: RoutineControlSubFunction,
    pub routine_id: RoutineIdentifier,
    pub data: Option<Vec<u8>>,
}

impl<RoutineIdentifier: IterableWireFormat> RoutineControlRequest<RoutineIdentifier> {
    pub(crate) fn new(
        sub_function: RoutineControlSubFunction,
        routine_id: RoutineIdentifier,
        data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            sub_function,
            routine_id,
            data,
        }
    }
}

impl<RoutineIdentifier: IterableWireFormat> WireFormat
    for RoutineControlRequest<RoutineIdentifier>
{
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let sub_function = RoutineControlSubFunction::from(reader.read_u8()?);
        let routine_id = RoutineIdentifier::option_from_reader(reader)?.unwrap();
        let data = Vec::<u8>::option_from_reader(reader)?;
        Ok(Some(Self {
            sub_function,
            routine_id,
            data,
        }))
    }

    fn required_size(&self) -> usize {
        3 + match self.data {
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

impl<RoutineIdentifier: IterableWireFormat> SingleValueWireFormat
    for RoutineControlRequest<RoutineIdentifier>
{
}

/// RoutineStatusRecord is a variable length field that can contain the status of the routine
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse {
    pub data: Vec<u8>,
}

impl RoutineControlResponse {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
impl WireFormat for RoutineControlResponse {
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(Self { data }))
    }

    fn required_size(&self) -> usize {
        self.data.len()
    }

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_all(&self.data)?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for RoutineControlResponse {}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{iterable_identifier, Identifier};
    use byteorder::{BigEndian, WriteBytesExt};

    iterable_identifier!(u16);

    #[test]
    fn simple_request() {
        // Fake data: StartRoutine, RoutineID of 0x8606 for "Start O2 Sensor Heater Test" or something
        let bytes: [u8; 6] = [0x01, 0x00, 0x01, 0x02, 0x03, 0x04];
        let req: RoutineControlRequest<u16> =
            RoutineControlRequest::from_reader(&mut bytes.as_slice()).unwrap();

        assert_eq!(u8::from(req.sub_function), 0x01);
        assert_eq!(req.routine_id, 0x0001);
        let data = req.data.clone().unwrap();
        assert_eq!(data, vec![0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = req.to_writer(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.required_size());

        let new_req: RoutineControlRequest<u16> = RoutineControlRequest::new(
            RoutineControlSubFunction::StopRoutine,
            0x0002,
            Some(vec![]),
        );

        assert_eq!(new_req.sub_function, RoutineControlSubFunction::StopRoutine);
        assert_eq!(new_req.routine_id, 0x0002);
    }
}
