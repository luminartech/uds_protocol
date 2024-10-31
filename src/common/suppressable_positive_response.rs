//! UDS uses a bit masking technique called the Suppress Positive Response Message Indication Bit (SPRMIB) to indicate if a positive response message should be suppressed.
//! This module provides a generic implementation of the Suppress Positive Response Message Indication Bit (SPRMIB) for UDS subfunction enumerations.

/// Suppress Positive Response Message Indication Bit
const SPRMIB: u8 = 0x80;
/// Mask to recover value in byte with SPRMIB
const SPRMIB_VALUE_MASK: u8 = 0x7F;

/// `SuppressablePositiveResponse` is used to encapsulate subfunction enumerations that can also encode the response suppression bit
/// This eliminates bit masking logic from a number of subfunction enumerations
#[non_exhaustive]
pub struct SuppressablePositiveResponse<T: From<u8> + Into<u8>> {
    suppress_positive_response: bool,
    value: T,
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
