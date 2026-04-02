use crate::{Decode, Encode, Error, SingleValueWireFormat, WireFormat};
use byteorder_embedded_io::BigEndian;
use byteorder_embedded_io::io::{ReadBytesExt, WriteBytesExt};

/// Implement [`WireFormat`] and [`SingleValueWireFormat`] for unsigned integer primitives.
#[macro_export]
macro_rules! unsigned_primitive_wire_format {
    ( $($primitive:ty), * ) => {
        $(
        impl WireFormat for $primitive {
            fn required_size(&self) -> usize {
                std::mem::size_of::<$primitive>()
            }
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.write_uint128::<BigEndian>(u128::from(*self), self.required_size())?;
                Ok(self.required_size())
            }
        }
        impl SingleValueWireFormat for $primitive {
            fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
                let value: $primitive = reader
                    .read_uint128::<BigEndian>(std::mem::size_of::<$primitive>())?
                    .try_into()
                    .expect("Failed to convert value to the target primitive type");
                Ok(value)
            }
        }
    )*
    };
}

unsigned_primitive_wire_format!(u8, u16, u32, u64, u128);

/// Implement [`WireFormat`] and [`SingleValueWireFormat`] for signed integer primitives.
#[macro_export]
macro_rules! signed_primitive_wire_format {
    ( $($primitive:ty), * ) => {
        $(
        impl WireFormat for $primitive {
            fn required_size(&self) -> usize {
                std::mem::size_of::<$primitive>()
            }
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
                writer.write_int128::<BigEndian>(i128::from(*self), self.required_size())?;
                Ok(self.required_size())
            }
        }
        impl SingleValueWireFormat for $primitive {
            fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
                let value: $primitive = reader
                    .read_int128::<BigEndian>(std::mem::size_of::<$primitive>())?
                    .try_into()
                    .expect("Failed to convert value to the target primitive type");
                Ok(value)
            }
        }
    )*
    };
}

signed_primitive_wire_format!(i8, i16, i32, i64, i128);

/// Implement [`Encode`] and [`Decode`] for unsigned integer primitives (no_std-compatible).
macro_rules! unsigned_primitive_encode_decode {
    ( $($primitive:ty), * ) => {
        $(
        impl Encode for $primitive {
            fn encoded_size(&self) -> usize {
                core::mem::size_of::<$primitive>()
            }
            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                writer.write_all(&self.to_be_bytes()).map_err(Error::io)?;
                Ok(self.encoded_size())
            }
        }
        impl<'a> Decode<'a> for $primitive {
            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                const SIZE: usize = core::mem::size_of::<$primitive>();
                if buf.len() < SIZE {
                    return Err(Error::InsufficientData(SIZE));
                }
                let (head, tail) = buf.split_at(SIZE);
                let value = <$primitive>::from_be_bytes(head.try_into().unwrap());
                Ok((value, tail))
            }
        }
        )*
    };
}

unsigned_primitive_encode_decode!(u8, u16, u32, u64, u128);

/// Implement [`Encode`] and [`Decode`] for signed integer primitives (no_std-compatible).
macro_rules! signed_primitive_encode_decode {
    ( $($primitive:ty), * ) => {
        $(
        impl Encode for $primitive {
            fn encoded_size(&self) -> usize {
                core::mem::size_of::<$primitive>()
            }
            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                writer.write_all(&self.to_be_bytes()).map_err(Error::io)?;
                Ok(self.encoded_size())
            }
        }
        impl<'a> Decode<'a> for $primitive {
            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                const SIZE: usize = core::mem::size_of::<$primitive>();
                if buf.len() < SIZE {
                    return Err(Error::InsufficientData(SIZE));
                }
                let (head, tail) = buf.split_at(SIZE);
                let value = <$primitive>::from_be_bytes(head.try_into().unwrap());
                Ok((value, tail))
            }
        }
        )*
    };
}

signed_primitive_encode_decode!(i8, i16, i32, i64, i128);

impl Encode for f32 {
    fn encoded_size(&self) -> usize {
        4
    }
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&self.to_be_bytes()).map_err(Error::io)?;
        Ok(4)
    }
}

impl<'a> Decode<'a> for f32 {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 4 {
            return Err(Error::InsufficientData(4));
        }
        let (head, tail) = buf.split_at(4);
        let value = f32::from_be_bytes(head.try_into().unwrap());
        Ok((value, tail))
    }
}

impl Encode for f64 {
    fn encoded_size(&self) -> usize {
        8
    }
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&self.to_be_bytes()).map_err(Error::io)?;
        Ok(8)
    }
}

impl<'a> Decode<'a> for f64 {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 8 {
            return Err(Error::InsufficientData(8));
        }
        let (head, tail) = buf.split_at(8);
        let value = f64::from_be_bytes(head.try_into().unwrap());
        Ok((value, tail))
    }
}

impl WireFormat for f32 {
    fn required_size(&self) -> usize {
        4
    }
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_f32::<BigEndian>(*self)?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for f32 {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let value: f32 = reader.read_f32::<BigEndian>()?;
        Ok(value)
    }
}

impl WireFormat for f64 {
    fn required_size(&self) -> usize {
        8
    }
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_f64::<BigEndian>(*self)?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for f64 {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let value: f64 = reader.read_f64::<BigEndian>()?;
        Ok(value)
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

        let u8_byte = <u8 as SingleValueWireFormat>::decode(&mut reader).unwrap();
        assert_eq!(u8_byte, 0xFF);
        assert_eq!(u8_byte.required_size(), 1);

        let mut write_buffer = vec![];
        WireFormat::encode(&u8_byte, &mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u16() {
        // Read some bytes
        let data = vec![0xFF, 0x01];
        let mut reader = &data[..];

        let u16_byte = <u16 as SingleValueWireFormat>::decode(&mut reader).unwrap();
        assert_eq!(u16_byte, 0xFF01);
        assert_eq!(u16_byte.required_size(), 2);

        let mut write_buffer = vec![];
        WireFormat::encode(&u16_byte, &mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u32() {
        // Read some bytes
        let data = vec![0xFF, 0x20, 0x02, 0x01];
        let mut reader = &data[..];

        let u32_byte = <u32 as SingleValueWireFormat>::decode(&mut reader).unwrap();
        assert_eq!(u32_byte, 0xFF20_0201);
        assert_eq!(u32_byte.required_size(), 4);

        let mut write_buffer = vec![];
        WireFormat::encode(&u32_byte, &mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }

    #[test]
    fn test_u64() {
        // Read some bytes
        let data = vec![0xFF, 0x20, 0x02, 0x01, 0xFF, 0x20, 0x02, 0x01];
        let mut reader = &data[..];

        let u64_byte = <u64 as SingleValueWireFormat>::decode(&mut reader).unwrap();
        assert_eq!(u64_byte, 0xFF20_0201_FF20_0201);
        assert_eq!(u64_byte.required_size(), 8);

        let mut write_buffer = vec![];
        WireFormat::encode(&u64_byte, &mut write_buffer).unwrap();
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

        let u128_byte = <u128 as SingleValueWireFormat>::decode(&mut reader).unwrap();
        assert_eq!(u128_byte, 0xFF20_0201_FF20_0201_FF20_0201_FF20_0201);
        assert_eq!(u128_byte.required_size(), 16);

        let mut write_buffer = vec![];
        WireFormat::encode(&u128_byte, &mut write_buffer).unwrap();
        assert_eq!(write_buffer, data);
    }
}
