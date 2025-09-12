use crate::{Error, WireFormat};
use byteorder::{BigEndian, ByteOrder};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, Deserialize, Parser, PartialEq, Serialize, ToSchema)]
pub struct BCD4ByteLE {
    pub value: u32,
}

impl BCD4ByteLE {
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub(crate) fn from_be(bytes: [u8; 4]) -> Self {
        let value = BigEndian::read_u32(&bytes);
        Self { value }
    }
}

impl FromStr for BCD4ByteLE {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Input string should be a 32-bit base 10 value
        let value = s
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("Failed to parse BCD4ByteLE: {e}"))?;
        Ok(BCD4ByteLE::new(value))
    }
}

impl WireFormat for BCD4ByteLE {
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        Ok(Some(BCD4ByteLE::from_be(bytes)))
    }

    fn required_size(&self) -> usize {
        4
    }

    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let total_written = writer.write(&self.value.to_be_bytes())?;
        Ok(total_written)
    }
}

#[cfg(test)]
mod bcd_tests {
    use super::*;

    #[test]
    fn test_from_str_bcd_valid_values() {
        let valid_inputs = vec![
            ("0".to_string(), 0),
            ("0 ".to_string(), 0),
            (" 0".to_string(), 0),
            (" 0 ".to_string(), 0),
            ("12345".to_string(), 12_345),
            (u32::MAX.to_string(), 4_294_967_295),
        ];

        for (input, expected_value) in valid_inputs {
            let bcd = BCD4ByteLE::from_str(&input).unwrap();
            assert_eq!(bcd.value, expected_value, "Failed for input: {input}");
        }
    }

    #[test]
    fn test_from_str_bcd_invalid_values() {
        let invalid_inputs = vec![
            "-1",         // negative numbers are not allowed
            "abc",        // non-numeric string
            "4294967296", // value too large for u32
            " ",          // empty string
        ];

        for input in invalid_inputs {
            let result = BCD4ByteLE::from_str(input);
            assert!(result.is_err(), "Expected error for input: {input}");
        }
    }
}
