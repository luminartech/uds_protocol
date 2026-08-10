//! Test-only helpers shared across the crate.

use crate::Encode;

/// Assert that an [`Encode`] value writes exactly `encoded_size()` bytes — both the
/// returned count AND the number of bytes actually consumed from the writer.
///
/// **This says nothing about *which* bytes get written.** `encoded_size` is a provided method on
/// [`Encode`] that counts by encoding into a sink, and no type in this crate overrides it, so
/// every quantity compared here is derived from `encode` itself; the upstream `debug_assert!`
/// inside `encoded_size` already catches a count that disagrees with the bytes written. What is
/// left is a real but narrow guarantee: that `encode` is deterministic across two invocations,
/// that its count matches a write into a real slice, and that the value fits the helper's buffer.
///
/// A call to this is therefore **not** byte-correctness coverage. Assert the expected bytes at
/// the call site as well — the absence of that is what let a swapped pair of encoded fields, and
/// a transposed pair of format-identifier nibbles, pass the whole suite.
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
