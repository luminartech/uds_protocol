use crate::{Error, RoutineControlSubFunction, SingleValueWireFormat, WireFormat};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest {
    pub sub_function: RoutineControlSubFunction,
    pub routine_id: u16,
    pub data: Vec<u8>,
}

impl RoutineControlRequest {
    pub(crate) fn new(
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        data: Vec<u8>,
    ) -> Self {
        Self {
            sub_function,
            routine_id,
            data,
        }
    }
}
impl WireFormat for RoutineControlRequest {
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let sub_function = RoutineControlSubFunction::from(reader.read_u8()?);
        let routine_id = reader.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(Self {
            sub_function,
            routine_id,
            data,
        }))
    }

    fn required_size(&self) -> usize {
        3 + self.data.len()
    }

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.sub_function))?;
        writer.write_u16::<BigEndian>(self.routine_id)?;
        writer.write_all(&self.data)?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for RoutineControlRequest {}

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

    #[test]
    fn simple_request() {
        let bytes: [u8; 6] = [0x01, 0x00, 0x01, 0x02, 0x03, 0x04];
        let req = RoutineControlRequest::from_reader(&mut bytes.as_slice()).unwrap();

        assert_eq!(u8::from(req.sub_function), 0x01);
        assert_eq!(req.routine_id, 0x0001);
        assert_eq!(req.data, vec![0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = req.to_writer(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.required_size());

        let new_req: RoutineControlRequest =
            RoutineControlRequest::new(RoutineControlSubFunction::StopRoutine, 0x0002, vec![]);

        assert_eq!(new_req.sub_function, RoutineControlSubFunction::StopRoutine);
        assert_eq!(new_req.routine_id, 0x0002);
    }
}
