use crate::Error;
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// `CommunicationType` is used to specify the type of communication behavior to be modified.
///
/// TODO: Note that this implementation is incomplete and does not properly handle the behavior of the upper 4 bits of the field.
/// This implementation is a placeholder and will be updated in the future, which will also be a breaking API change.
///
/// Note:
///
/// Conversions from `u8` to `CommunicationType` are fallible and will return an [`Error`](crate::Error) if the value is not a valid `CommunicationType`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = crate::Error, constructor = Error::InvalidCommunicationType))]
#[repr(u8)]
pub enum CommunicationType {
    /// This value is reserved by the ISO 14229-1 Specification
    ISOSAEReserved = 0x00,
    /// This value represents all application related communication.
    Normal = 0x01,
    /// This value represents all network management related communication.
    NetworkManagement = 0x02,
    /// This value represents all application and network management related communication.
    NormalAndNetworkManagement = 0x03,
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
