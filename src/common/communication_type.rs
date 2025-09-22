use clap::ValueEnum;

use crate::Error;

/// `CommunicationType` is used to specify the type of communication behavior to be modified.
///
/// TODO: Note that this implementation is incomplete and does not properly handle the behavior of the upper 4 bits of the field.
/// This implementation is a placeholder and will be updated in the future, which will also be a breaking API change.
///
/// Note:
///
/// Conversions from `u8` to `CommunicationType` are fallible and will return an [`Error`](crate::Error) if the value is not a valid `CommunicationType`
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Clone, Copy, Debug, Eq, PartialEq,  ValueEnum, utoipa::ToSchema,
)]
pub enum CommunicationType {
    /// This value is reserved by the ISO 14229-1 Specification
    ISOSAEReserved,
    /// This value represents all application related communication.
    Normal,
    /// This value represents all network management related communication.
    NetworkManagement,
    /// This value represents all application and network management related communication.
    NormalAndNetworkManagement,
}

impl From<CommunicationType> for u8 {
    fn from(value: CommunicationType) -> Self {
        match value {
            CommunicationType::ISOSAEReserved => 0x00,
            CommunicationType::Normal => 0x01,
            CommunicationType::NetworkManagement => 0x02,
            CommunicationType::NormalAndNetworkManagement => 0x03,
        }
    }
}

impl TryFrom<u8> for CommunicationType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::ISOSAEReserved),
            0x01 => Ok(Self::Normal),
            0x02 => Ok(CommunicationType::NetworkManagement),
            0x03 => Ok(CommunicationType::NormalAndNetworkManagement),
            val => Err(Error::InvalidCommunicationType(val)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn communication_type_from_all_u8_values() {
        for i in 0..=u8::MAX {
            let msg_type = CommunicationType::try_from(i);
            match i {
                0x00 => assert!(matches!(msg_type, Ok(CommunicationType::ISOSAEReserved))),
                0x01 => assert!(matches!(msg_type, Ok(CommunicationType::Normal))),
                0x02 => assert!(matches!(msg_type, Ok(CommunicationType::NetworkManagement))),
                0x03 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationType::NormalAndNetworkManagement)
                )),
                _ => assert!(matches!(msg_type, Err(Error::InvalidCommunicationType(_)))),
            }
        }
    }

    #[test]
    fn communication_type_round_trip_all_values() {
        for i in 0..=u8::MAX {
            let value = CommunicationType::try_from(i);
            match value {
                Ok(value) => assert_eq!(u8::from(value), i),
                Err(Error::InvalidCommunicationType(value)) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }
}
