use crate::{Decode, Encode, Error, Incomplete};

/// Implement [`Encode`] and [`Decode`] for integer primitives (no_std-compatible).
macro_rules! primitive_encode_decode {
    ( $($primitive:ty), * ) => {
        $(
        impl Encode for $primitive {
            fn encoded_size(&self) -> usize {
                core::mem::size_of::<$primitive>()
            }
            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                writer.write_all(&self.to_be_bytes()).map_err(Error::io)?;
                Ok(core::mem::size_of::<$primitive>())
            }
        }
        impl<'a> Decode<'a> for $primitive {
            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                const SIZE: usize = core::mem::size_of::<$primitive>();
                if buf.len() < SIZE {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: SIZE,
                        available: buf.len(),
                    }));
                }
                let (head, tail) = buf.split_at(SIZE);
                let value = <$primitive>::from_be_bytes(head.try_into().unwrap());
                Ok((value, tail))
            }
        }
        )*
    };
}

primitive_encode_decode!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

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
            return Err(Error::InsufficientData(Incomplete {
                needed: 4,
                available: buf.len(),
            }));
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
            return Err(Error::InsufficientData(Incomplete {
                needed: 8,
                available: buf.len(),
            }));
        }
        let (head, tail) = buf.split_at(8);
        let value = f64::from_be_bytes(head.try_into().unwrap());
        Ok((value, tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_encode_decode() {
        let val: u8 = 0xFF;
        let mut buf = [0u8; 1];
        Encode::encode(&val, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(buf, [0xFF]);
        let (decoded, rest) = <u8 as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, 0xFF);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_u16_encode_decode() {
        let val: u16 = 0xFF01;
        let mut buf = [0u8; 2];
        Encode::encode(&val, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(buf, [0xFF, 0x01]);
        let (decoded, rest) = <u16 as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, 0xFF01);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_u32_encode_decode() {
        let val: u32 = 0xFF20_0201;
        let mut buf = [0u8; 4];
        Encode::encode(&val, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(buf, [0xFF, 0x20, 0x02, 0x01]);
        let (decoded, rest) = <u32 as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, 0xFF20_0201);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_u64_encode_decode() {
        let val: u64 = 0xFF20_0201_FF20_0201;
        let mut buf = [0u8; 8];
        Encode::encode(&val, &mut buf.as_mut_slice()).unwrap();
        let (decoded, rest) = <u64 as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, val);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_u128_encode_decode() {
        let val: u128 = 0xFF20_0201_FF20_0201_FF20_0201_FF20_0201;
        let mut buf = [0u8; 16];
        Encode::encode(&val, &mut buf.as_mut_slice()).unwrap();
        let (decoded, rest) = <u128 as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, val);
        assert!(rest.is_empty());
    }
}
