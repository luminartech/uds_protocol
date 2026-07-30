//! Test-only helpers shared across the crate.

use crate::Encode;

/// Assert that an [`Encode`] value writes exactly `encoded_size()` bytes — both the
/// returned count AND the number of bytes actually consumed from the writer.
///
/// Guards against `encode` and `encoded_size` drifting, and against `encode` returning
/// a count that disagrees with how many bytes it actually wrote — either corrupts callers
/// that pre-size a buffer from `encoded_size()`.
pub(crate) fn assert_encode_size_agrees<T: Encode>(value: &T)
where
    T::Error: core::fmt::Debug,
{
    let mut buf = [0u8; 512];
    let cap = buf.len();
    let size = value.encoded_size().unwrap();
    assert!(
        size <= cap,
        "test helper buffer too small: encoded_size() is {size}, buffer is {cap}"
    );
    let mut writer: &mut [u8] = &mut buf;
    let written = value.encode(&mut writer).unwrap();
    let consumed = cap - writer.len();
    assert_eq!(
        written, size,
        "encode returned {written}, encoded_size() is {size}"
    );
    assert_eq!(
        consumed, size,
        "encode consumed {consumed} bytes, encoded_size() is {size}"
    );
}

/// Compile-time assertion that `T: Eq`. Never called at runtime; instantiating it
/// in a test forces a compile error until the type derives `Eq`.
#[allow(dead_code)]
pub(crate) const fn assert_impl_eq<T: Eq>() {}

/// Compile-time assertion that `T` round-trips serde (borrowed deserialize allowed).
#[cfg(feature = "serde")]
#[allow(dead_code)]
pub(crate) const fn assert_impl_serde<'de, T: serde::Serialize + serde::Deserialize<'de>>() {}
