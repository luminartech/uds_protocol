use crate::Error;

/// Security Access Type allows for multiple different security challenges within an ECU.
///
/// The Security Access Type is used to determine both the sub function,
/// as well as ECU specific access type being requested
///
/// *Note*:
///
/// Conversions from `u8` to `SecurityAccessType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityAccessType {
    /// This value is reserved for future definition
    ISOSAEReserved(u8),
    /// `RequestSeed` with the level of security defined by the vehicle manufacturer
    RequestSeed(u8),
    /// `SendKey` with the level of security defined by the vehicle manufacturer
    SendKey(u8),
    /// `RequestSeed` with different levels of security defined for end of life
    /// activation of on-board pyrotechnic devices
    ISO26021_2Values,
    /// `SendKey` with different levels of security defined for end of life activation
    ISO26021_2SendKeyValues,
    /// This range of values is reserved for system supplier specific use
    SystemSupplierSpecific(u8),
}

impl From<SecurityAccessType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: SecurityAccessType) -> Self {
        match value {
            SecurityAccessType::ISOSAEReserved(val) => val,
            SecurityAccessType::RequestSeed(val) => val,
            SecurityAccessType::SendKey(val) => val,
            SecurityAccessType::ISO26021_2Values => 0x5F,
            SecurityAccessType::ISO26021_2SendKeyValues => 0x60,
            SecurityAccessType::SystemSupplierSpecific(val) => val,
        }
    }
}

impl TryFrom<u8> for SecurityAccessType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 | 0x43..=0x5E | 0x7F => Ok(Self::ISOSAEReserved(value)),
            // Security requests alternate, with odd numbers being seed requests,
            // and even numbers being send key requests
            0x01..=0x42 => {
                if value % 2 == 1 {
                    Ok(Self::RequestSeed(value))
                } else {
                    Ok(Self::SendKey(value))
                }
            }
            0x5F => Ok(Self::ISO26021_2Values),
            0x60 => Ok(Self::ISO26021_2SendKeyValues),
            0x61..=0x7E => Ok(Self::SystemSupplierSpecific(value)),
            _ => Err(Error::InvalidSecurityAccessType(value)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const REQUEST_SEED_VALUES: [u8; 33] = [
        0x01, 0x03, 0x05, 0x07, 0x09, 0x0B, 0x0D, 0x0F, 0x11, 0x13, 0x15, 0x17, 0x19, 0x1B, 0x1D,
        0x1F, 0x21, 0x23, 0x25, 0x27, 0x29, 0x2B, 0x2D, 0x2F, 0x31, 0x33, 0x35, 0x37, 0x39, 0x3B,
        0x3D, 0x3F, 0x41,
    ];
    const SEND_KEY_VALUES: [u8; 33] = [
        0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E,
        0x20, 0x22, 0x24, 0x26, 0x28, 0x2A, 0x2C, 0x2E, 0x30, 0x32, 0x34, 0x36, 0x38, 0x3A, 0x3C,
        0x3E, 0x40, 0x42,
    ];
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn security_access_type_from_all_u8_values() {
        assert_eq!(
            SecurityAccessType::try_from(0).unwrap(),
            SecurityAccessType::ISOSAEReserved(0)
        );
        for value in &REQUEST_SEED_VALUES {
            assert_eq!(
                SecurityAccessType::try_from(*value).unwrap(),
                SecurityAccessType::RequestSeed(*value)
            );
        }
        for value in &SEND_KEY_VALUES {
            assert_eq!(
                SecurityAccessType::try_from(*value).unwrap(),
                SecurityAccessType::SendKey(*value)
            );
        }
        for i in 0x43..=0x5E {
            assert_eq!(
                SecurityAccessType::try_from(i).unwrap(),
                SecurityAccessType::ISOSAEReserved(i)
            );
        }
        assert_eq!(
            SecurityAccessType::try_from(0x5F).unwrap(),
            SecurityAccessType::ISO26021_2Values
        );
        assert_eq!(
            SecurityAccessType::try_from(0x60).unwrap(),
            SecurityAccessType::ISO26021_2SendKeyValues
        );
        for i in 0x61..=0x7E {
            assert_eq!(
                SecurityAccessType::try_from(i).unwrap(),
                SecurityAccessType::SystemSupplierSpecific(i)
            );
        }
        assert_eq!(
            SecurityAccessType::try_from(0x7F).unwrap(),
            SecurityAccessType::ISOSAEReserved(0x7F)
        );
        for i in 0x80..=0xFF {
            match SecurityAccessType::try_from(i).unwrap_err() {
                Error::InvalidSecurityAccessType(value) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }

    #[test]
    fn security_access_type_round_trip_all_values() {
        for i in 0..=u8::MAX {
            let value = SecurityAccessType::try_from(i);
            match value {
                Ok(value) => assert_eq!(u8::from(value), i),
                Err(Error::InvalidSecurityAccessType(value)) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }
}
