//! UDS uses a bit masking technique called the Suppress Positive Response Message Indication Bit (SPRMIB) to indicate if a positive response message should be suppressed.
//! This module provides a generic implementation of the Suppress Positive Response Message Indication Bit (SPRMIB) for UDS subfunction enumerations.

use serde::{Deserialize, Serialize};

/// Suppress Positive Response Message Indication Bit
const SPRMIB: u8 = 0x80;
/// Mask to recover value in byte with SPRMIB
pub(crate) const SPRMIB_VALUE_MASK: u8 = 0x7F;

/// `SuppressablePositiveResponse` is used to encapsulate subfunction enumerations that can also encode the response suppression bit
/// This eliminates bit masking logic from a number of subfunction enumerations
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct SuppressablePositiveResponse<T: From<u8> + Into<u8>> {
    suppress_positive_response: bool,
    value: T,
}

impl<T: From<u8> + Into<u8>> SuppressablePositiveResponse<T> {
    /// Returns a new `SuppressablePositiveResponse` with the given value and suppression flag
    pub const fn new(suppress_positive_response: bool, value: T) -> Self {
        Self {
            suppress_positive_response,
            value,
        }
    }

    /// Returns the value of the `SuppressablePositiveResponse`
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the suppression flag of the `SuppressablePositiveResponse`
    pub const fn suppress_positive_response(&self) -> bool {
        self.suppress_positive_response
    }
}

impl<T: From<u8> + Into<u8>> From<u8> for SuppressablePositiveResponse<T> {
    fn from(value: u8) -> Self {
        let suppress_positive_response = value & SPRMIB == SPRMIB;
        let value = T::from(value & SPRMIB_VALUE_MASK);
        Self {
            suppress_positive_response,
            value,
        }
    }
}

impl<T: From<u8> + Into<u8>> From<SuppressablePositiveResponse<T>> for u8 {
    fn from(value: SuppressablePositiveResponse<T>) -> Self {
        let mut result = value.value.into();
        if value.suppress_positive_response {
            result |= SPRMIB;
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn test_all_u8_values() {
        for i in 0..=u8::MAX {
            let value = SuppressablePositiveResponse::<u8>::from(i);
            match i {
                0x00..=0x7F => {
                    assert_eq!(value.value, i);
                    assert!(!value.suppress_positive_response);
                }
                0x80..=0xFF => {
                    assert_eq!(value.value, i & SPRMIB_VALUE_MASK);
                    assert!(value.suppress_positive_response);
                }
            }
        }
    }
}
