use clap::ValueEnum;

use crate::Error;

/// `CommunicationControlType` is used to specify the type of communication behavior to be modified
///
/// *Note*:
///
/// Conversions from `u8` to `CommunicationControlType` are fallible and will return an [`Error`](crate::Error) if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum CommunicationControlType {
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`](crate::CommunicationType)
    EnableRxAndTx,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`](crate::CommunicationType)
    EnableRxAndDisableTx,
    /// This value indicates that the reception of messages shall be disabled
    /// and the transmission of messages shall be enabled for the specified [`CommunicationType`](crate::CommunicationType)
    DisableRxAndEnableTx,
    /// This value indicates that the reception and transmission of messages
    /// shall be disabled for the specified [`CommunicationType`](crate::CommunicationType)
    DisableRxAndTx,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`](crate::CommunicationType)
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndDisableTxWithEnhancedAddressInfo,
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`](crate::CommunicationType)
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndTxWithEnhancedAddressInfo,
    /// These values are reserved by the ISO 14229-1 Specification
    #[clap(skip)]
    ISOSAEReserved(u8),
    /// Values reserved for use by vehicle manufacturers
    #[clap(skip)]
    VehicleManufacturerSpecific(u8),
    /// Values reserved for use by system suppliers
    #[clap(skip)]
    SystemSupplierSpecific(u8),
}

impl CommunicationControlType {
    #[must_use]
    pub const fn is_extended_address_variant(&self) -> bool {
        matches!(
            self,
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo
                | CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo
        )
    }
}

impl From<CommunicationControlType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: CommunicationControlType) -> Self {
        match value {
            CommunicationControlType::EnableRxAndTx => 0x00,
            CommunicationControlType::EnableRxAndDisableTx => 0x01,
            CommunicationControlType::DisableRxAndEnableTx => 0x02,
            CommunicationControlType::DisableRxAndTx => 0x03,
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo => 0x04,
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo => 0x05,
            CommunicationControlType::ISOSAEReserved(val) => val,
            CommunicationControlType::VehicleManufacturerSpecific(val) => val,
            CommunicationControlType::SystemSupplierSpecific(val) => val,
        }
    }
}

impl TryFrom<u8> for CommunicationControlType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::EnableRxAndTx),
            0x01 => Ok(Self::EnableRxAndDisableTx),
            0x02 => Ok(Self::DisableRxAndEnableTx),
            0x03 => Ok(Self::DisableRxAndTx),
            0x04 => Ok(Self::EnableRxAndDisableTxWithEnhancedAddressInfo),
            0x05 => Ok(Self::EnableRxAndTxWithEnhancedAddressInfo),
            0x06..=0x3F | 0x7F => Ok(Self::ISOSAEReserved(value)),
            0x40..=0x5F => Ok(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Ok(Self::SystemSupplierSpecific(value)),
            _ => Err(Error::InvalidCommunicationControlType(value)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn from_all_u8_values() {
        for i in 0..=u8::MAX {
            let msg_type = CommunicationControlType::try_from(i);
            match i {
                0x00 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndTx)
                )),
                0x01 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndDisableTx)
                )),
                0x02 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::DisableRxAndEnableTx)
                )),
                0x03 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::DisableRxAndTx)
                )),
                0x04 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo)
                )),
                0x05 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo)
                )),
                0x06..=0x3F | 0x7F => {
                    assert!(matches!(
                        msg_type,
                        Ok(CommunicationControlType::ISOSAEReserved(_))
                    ));
                }
                0x40..=0x5F => {
                    assert!(matches!(
                        msg_type,
                        Ok(CommunicationControlType::VehicleManufacturerSpecific(_))
                    ));
                }
                0x60..=0x7E => {
                    assert!(matches!(
                        msg_type,
                        Ok(CommunicationControlType::SystemSupplierSpecific(_))
                    ));
                }
                _ => assert!(matches!(
                    msg_type,
                    Err(Error::InvalidCommunicationControlType(_))
                )),
            }
        }
    }

    #[test]
    fn communication_control_type_round_trip_all_values() {
        for i in 0..=u8::MAX {
            let value = CommunicationControlType::try_from(i);
            match value {
                Ok(value) => assert_eq!(u8::from(value), i),
                Err(Error::InvalidCommunicationControlType(value)) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }
}
