//! Compute the number of bytes needed to represent a value using core
pub fn param_length_u128(value: u128) -> u16 {
    ((u128::BITS - value.leading_zeros() + 7) / 8) as u16
}
pub fn param_length_u64(value: u64) -> u8 {
    ((u64::BITS - value.leading_zeros() + 7) / 8) as u8
}
pub fn param_length_u32(value: u32) -> u8 {
    ((u32::BITS - value.leading_zeros() + 7) / 8) as u8
}
pub fn param_length_u16(value: u16) -> u8 {
    ((u16::BITS - value.leading_zeros() + 7) / 8) as u8
}

/// Identifiers are (generally) u16 values that are used to identify a specific piece of data or routine
///
/// Macro used to implement the [WireFormat] and [IterableWireFormat] traits for a type that implements the [Identifier] trait
#[macro_export]
macro_rules! iterable_identifier {
    ($type:ty) => {
        #[doc = "Implement the [WireFormat] and [IterableWireFormat] traits for $type"]
        impl WireFormat for $type
        where
            Self: Identifier,
        {
            fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
                let mut identifier_data: [u8; 2] = [0; 2];
                match reader.read(&mut identifier_data)? {
                    0 => return Ok(None),
                    1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
                    2 => (),
                    _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
                };

                match Self::try_from(u16::from_be_bytes(identifier_data)) {
                    Ok(identifier) => Ok(Some(identifier)),
                    Err(_) => Err(Error::InvalidDiagnosticIdentifier(u16::from_be_bytes(
                        identifier_data,
                    ))),
                }
            }

            fn required_size(&self) -> usize {
                2
            }

            fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
                writer.write_u16::<BigEndian>(u16::from(*self))?;
                Ok(2)
            }
        }
        impl IterableWireFormat for $type {}
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bits_needed() {
        assert_eq!(param_length_u32(0x1234), 2);
        assert_eq!(param_length_u16(1u16), 1);
        assert_eq!(param_length_u16(2u16), 1);
        assert_eq!(param_length_u16(3u16), 1);
        assert_eq!(param_length_u16(4u16), 1);
        assert_eq!(param_length_u16(7u16), 1);
        assert_eq!(param_length_u16(8u16), 1);
        assert_eq!(param_length_u16(15u16), 1);
        assert_eq!(param_length_u16(16u16), 1);

        // Test with different unsigned types
        assert_eq!(param_length_u32(0u32), 0);
        assert_eq!(param_length_u32(0x1_FFFF), 3);
        assert_eq!(param_length_u64(900000), 3);
        assert_eq!(param_length_u128(137439853472), 5);
    }
}
