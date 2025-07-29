//! Compute the number of bytes needed to represent a value using core
pub fn param_length_u128(value: u128) -> u16 {
    (u128::BITS - value.leading_zeros()).div_ceil(8) as u16
}
pub fn param_length_u64(value: u64) -> u8 {
    (u64::BITS - value.leading_zeros()).div_ceil(8) as u8
}
pub fn param_length_u32(value: u32) -> u8 {
    (u32::BITS - value.leading_zeros()).div_ceil(8) as u8
}
pub fn param_length_u16(value: u16) -> u8 {
    (u16::BITS - value.leading_zeros()).div_ceil(8) as u8
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
