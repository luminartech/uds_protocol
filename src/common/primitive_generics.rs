use crate::{Error, WireFormat};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[macro_export]
macro_rules! unsigned_primitive_wire_format {
    ( $($primitive:ty), * ) => {
        $(
        impl WireFormat for $primitive {
            fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
                let value: $primitive = reader
                    .read_uint128::<BigEndian>(std::mem::size_of::<$primitive>())?
                    .try_into()
                    .expect("Failed to convert value to the target primitive type");
                Ok(Some(value))
            }
            fn required_size(&self) -> usize {
                std::mem::size_of::<$primitive>()
            }
            fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.write_uint128::<BigEndian>(u128::from(*self), self.required_size())?;
                Ok(self.required_size())
            }
        }
    )*
    };
}

unsigned_primitive_wire_format!(u8, u16, u32, u64, u128);

#[macro_export]
macro_rules! signed_primitive_wire_format {
    ( $($primitive:ty), * ) => {
        $(
        impl WireFormat for $primitive {
            fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
                let value: $primitive = reader
                    .read_int128::<BigEndian>(std::mem::size_of::<$primitive>())?
                    .try_into()
                    .expect("Failed to convert value to the target primitive type");
                Ok(Some(value))
            }
            fn required_size(&self) -> usize {
                std::mem::size_of::<$primitive>()
            }
            fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.write_int128::<BigEndian>(i128::from(*self), self.required_size())?;
                Ok(self.required_size())
            }
        }
    )*
    };
}

signed_primitive_wire_format!(i8, i16, i32, i64, i128);

impl WireFormat for f32 {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value: f32 = reader.read_f32::<BigEndian>()?;
        Ok(Some(value))
    }
    fn required_size(&self) -> usize {
        4
    }
    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_f32::<BigEndian>(*self)?;
        Ok(self.required_size())
    }
}

impl WireFormat for f64 {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let value: f64 = reader.read_f64::<BigEndian>()?;
        Ok(Some(value))
    }
    fn required_size(&self) -> usize {
        8
    }
    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_f64::<BigEndian>(*self)?;
        Ok(self.required_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8() {
        // Read some bytes
        let data = vec![0xFF];
        let mut reader = &data[..];

        let u8_byte = u8::option_from_reader(&mut reader).unwrap().unwrap();
        assert_eq!(u8_byte, 0xFF);
        assert_eq!(u8_byte.required_size(), 1);

        let mut write_buffer = vec![];
        u8_byte.to_writer(&mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u16() {
        // Read some bytes
        let data = vec![0xFF, 0x01];
        let mut reader = &data[..];

        let u16_byte = u16::option_from_reader(&mut reader).unwrap().unwrap();
        assert_eq!(u16_byte, 0xFF01);
        assert_eq!(u16_byte.required_size(), 2);

        let mut write_buffer = vec![];
        u16_byte.to_writer(&mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u32() {
        // Read some bytes
        let data = vec![0xFF, 0x20, 0x02, 0x01];
        let mut reader = &data[..];

        let u32_byte = u32::option_from_reader(&mut reader).unwrap().unwrap();
        assert_eq!(u32_byte, 0xFF200201);
        assert_eq!(u32_byte.required_size(), 4);

        let mut write_buffer = vec![];
        u32_byte.to_writer(&mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u64() {
        // Read some bytes
        let data = vec![0xFF, 0x20, 0x02, 0x01, 0xFF, 0x20, 0x02, 0x01];
        let mut reader = &data[..];

        let u64_byte = u64::option_from_reader(&mut reader).unwrap().unwrap();
        assert_eq!(u64_byte, 0xFF200201FF200201);
        assert_eq!(u64_byte.required_size(), 8);

        let mut write_buffer = vec![];
        u64_byte.to_writer(&mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u128() {
        // Read some bytes
        let data = vec![
            0xFF, 0x20, 0x02, 0x01, 0xFF, 0x20, 0x02, 0x01, 0xFF, 0x20, 0x02, 0x01, 0xFF, 0x20,
            0x02, 0x01,
        ];
        let mut reader = &data[..];

        let u128_byte = u128::option_from_reader(&mut reader).unwrap().unwrap();
        assert_eq!(u128_byte, 0xFF200201FF200201FF200201FF200201);
        assert_eq!(u128_byte.required_size(), 16);

        let mut write_buffer = vec![];
        u128_byte.to_writer(&mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }
}
