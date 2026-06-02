//! Test-only helpers shared across the crate.

use crate::Encode;

/// Assert that an [`Encode`] value writes exactly `encoded_size()` bytes.
///
/// Guards against the two methods drifting, which would corrupt callers that pre-size
/// a buffer from `encoded_size()`.
pub(crate) fn assert_encode_size_agrees<T: Encode>(value: &T) {
    let mut buf = [0u8; 512];
    let mut writer = buf.as_mut_slice();
    let written = value.encode(&mut writer).unwrap();
    assert_eq!(
        written,
        value.encoded_size(),
        "encode wrote {written} bytes but encoded_size() reported {}",
        value.encoded_size()
    );
}
