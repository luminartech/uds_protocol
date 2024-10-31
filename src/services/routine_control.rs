use crate::{Error, RoutineControlSubFunction};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let sub_function = RoutineControlSubFunction::from(buffer.read_u8()?);
        let routine_id = buffer.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Self {
            sub_function,
            routine_id,
            data,
        })
    }
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.sub_function))?;
        buffer.write_u16::<BigEndian>(self.routine_id)?;
        buffer.write_all(&self.data)?;
        Ok(())
    }
}
