//! Variable-width big-endian integer helpers ([`read_be_uint`]/[`write_be_uint`]) and the
//! `param_length_*` functions that compute the minimum number of bytes needed to represent
//! a value — all `core`-only (no `std`/`alloc`).
use crate::Error;

/// Maximum width of a big-endian unsigned integer this codec handles.
const BE_UINT_MAX_BYTES: usize = 16;

/// Read the first `n` big-endian bytes of `src` as a left-padded `u128`.
///
/// # Errors
/// Returns [`Error::InsufficientData`] if `src` is shorter than `n`, or
/// [`Error::IncorrectMessageLengthOrInvalidFormat`] if `n > 16`.
pub(crate) fn read_be_uint(src: &[u8], n: usize) -> Result<u128, Error> {
    if n > BE_UINT_MAX_BYTES {
        return Err(Error::IncorrectMessageLengthOrInvalidFormat);
    }
    if src.len() < n {
        return Err(Error::InsufficientData(n));
    }
    let mut bytes = [0u8; BE_UINT_MAX_BYTES];
    bytes[BE_UINT_MAX_BYTES - n..].copy_from_slice(&src[..n]);
    Ok(u128::from_be_bytes(bytes))
}

/// Write the low `n` big-endian bytes of `value` to `writer`, returning `n`.
/// Only the low `n` bytes of `value` are written; higher bytes are discarded.
///
/// # Errors
/// Returns [`Error::IncorrectMessageLengthOrInvalidFormat`] if `n > 16`, or
/// [`Error::IoError`] if the writer fails.
pub(crate) fn write_be_uint(
    value: u128,
    n: usize,
    writer: &mut impl embedded_io::Write,
) -> Result<usize, Error> {
    if n > BE_UINT_MAX_BYTES {
        return Err(Error::IncorrectMessageLengthOrInvalidFormat);
    }
    let bytes = value.to_be_bytes();
    writer
        .write_all(&bytes[BE_UINT_MAX_BYTES - n..])
        .map_err(Error::io)?;
    Ok(n)
}

/// Return the minimum number of bytes needed to represent a `u16` value.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn param_length_u16(value: u16) -> u8 {
    (u16::BITS - value.leading_zeros()).div_ceil(8) as u8
}
/// Return the minimum number of bytes needed to represent a `u32` value.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn param_length_u32(value: u32) -> u8 {
    (u32::BITS - value.leading_zeros()).div_ceil(8) as u8
}
/// Return the minimum number of bytes needed to represent a `u64` value.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn param_length_u64(value: u64) -> u8 {
    (u64::BITS - value.leading_zeros()).div_ceil(8) as u8
}
/// Return the minimum number of bytes needed to represent a `u128` value.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn param_length_u128(value: u128) -> u16 {
    (u128::BITS - value.leading_zeros()).div_ceil(8) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn be_uint_roundtrip() {
        use crate::shared::util::{read_be_uint, write_be_uint};
        let mut buf = [0u8; 16];
        let mut w = buf.as_mut_slice();
        let written = write_be_uint(0x00AB_CDEFu128, 3, &mut w).unwrap();
        assert_eq!(written, 3);
        assert_eq!(&buf[..3], &[0xAB, 0xCD, 0xEF]);
        let v = read_be_uint(&buf[..3], 3).unwrap();
        assert_eq!(v, 0x00AB_CDEF);
    }

    #[test]
    fn be_uint_zero_width() {
        use crate::shared::util::{read_be_uint, write_be_uint};
        let mut buf = [0u8; 4];
        let mut w = buf.as_mut_slice();
        assert_eq!(write_be_uint(0, 0, &mut w).unwrap(), 0);
        assert_eq!(read_be_uint(&[], 0).unwrap(), 0);
    }

    #[test]
    fn read_be_uint_rejects_short_and_overwide() {
        use crate::shared::util::read_be_uint;
        assert!(read_be_uint(&[0x01], 2).is_err());
        assert!(read_be_uint(&[0u8; 17], 17).is_err());
    }

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
        assert_eq!(param_length_u64(900_000), 3);
        assert_eq!(param_length_u128(137_439_853_472), 5);
    }
}
