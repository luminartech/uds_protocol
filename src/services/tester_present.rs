use crate::Error;

use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TesterPresentRequest {
    zero_sub_function: u8,
}

impl TesterPresentRequest {
    pub(crate) fn new() -> Self {
        Self {
            zero_sub_function: 0,
        }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let zero_sub_function = buffer.read_u8()?;
        Ok(Self { zero_sub_function })
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.zero_sub_function))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_type() {
        let bytes = vec![0 as u8];
        let mut byte_access = Cursor::new(bytes);
        let test_type = TesterPresentRequest::read(&mut byte_access).unwrap();
        assert_eq!(test_type, TesterPresentRequest::new());
    }

    #[test]
    fn write_type() {
        let test_type = TesterPresentRequest::new();
        let mut buffer = Vec::new();
        test_type.write(&mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
    }
}
