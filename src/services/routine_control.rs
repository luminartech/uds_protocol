use crate::{Error, RoutineControlSubFunction, SingleValueWireFormat, WireFormat};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Deserialize;
use std::io::{Read, Write};

#[derive(Clone, Debug, Deserialize)]
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
impl WireFormat<'_, Error> for RoutineControlRequest {
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

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.sub_function))?;
        writer.write_u16::<BigEndian>(self.routine_id)?;
        writer.write_all(&self.data)?;
        Ok(3 + self.data.len())
    }
}

impl SingleValueWireFormat<'_, Error> for RoutineControlRequest {}
