//! UDS uses a bit masking technique called the Suppress Positive Response Message Indication Bit (SPRMIB) to indicate if a positive response message should be suppressed.
//! This module provides a generic implementation of the Suppress Positive Response Message Indication Bit (SPRMIB) for UDS subfunction enumerations.
use crate::Error;

/// Suppress Positive Response Message Indication Bit
const SPRMIB: u8 = 0x80;
/// Mask to recover value in byte with SPRMIB
pub(crate) const SPRMIB_VALUE_MASK: u8 = 0x7F;

/// `SuppressablePositiveResponse` is used to encapsulate subfunction enumerations that can also encode the response suppression bit.
/// This eliminates bit masking logic from a number of subfunction enumerations.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Clone, Copy, Debug, Eq, PartialEq,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[non_exhaustive]
pub(crate) struct SuppressablePositiveResponse<T: TryFrom<u8> + Into<u8> + Copy> {
    suppress_positive_response: bool,
    value: T,
}

impl<T: TryFrom<u8> + Into<u8> + Copy> SuppressablePositiveResponse<T> {
    /// Returns a new `SuppressablePositiveResponse` with the given value and suppression flag
    pub const fn new(suppress_positive_response: bool, value: T) -> Self {
        Self {
            suppress_positive_response,
            value,
        }
    }

    /// Returns the value of the `SuppressablePositiveResponse`
    pub const fn value(&self) -> T {
        self.value
    }

    /// Returns the suppression flag of the `SuppressablePositiveResponse`
    pub const fn suppress_positive_response(&self) -> bool {
        self.suppress_positive_response
    }
}

impl<T: TryFrom<u8, Error = Error> + Into<u8> + Copy> TryFrom<u8>
    for SuppressablePositiveResponse<T>
{
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        let suppress_positive_response = value & SPRMIB == SPRMIB;
        let value = T::try_from(value & SPRMIB_VALUE_MASK)?;
        Ok(Self {
            suppress_positive_response,
            value,
        })
    }
}

impl<T: TryFrom<u8> + Into<u8> + Copy> From<SuppressablePositiveResponse<T>> for u8 {
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
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Testu8(u8);
    impl TryFrom<u8> for Testu8 {
        type Error = crate::Error;
        fn try_from(value: u8) -> Result<Self, crate::Error> {
            Ok(Self(value))
        }
    }
    impl From<Testu8> for u8 {
        fn from(test: Testu8) -> Self {
            test.0
        }
    }
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn test_all_u8_values() {
        for i in 0..=u8::MAX {
            let value: SuppressablePositiveResponse<Testu8> =
                SuppressablePositiveResponse::try_from(i).unwrap();
            if let 0x00..=0x7F = i {
                assert_eq!(value.value().0, i);
                assert!(!value.suppress_positive_response());
            } else {
                assert_eq!(value.value().0, i & SPRMIB_VALUE_MASK);
                assert!(value.suppress_positive_response());
            }
        }
    }
}
