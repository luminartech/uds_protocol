use crate::Error;

/// `CommunicationControlType` is used to specify the type of communication behavior to be modified
///
/// *Note*:
///
/// Conversions from `u8` to `CommunicationControlType` are fallible and will return an [`Error`](crate::Error) if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CommunicationControlType {
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`](crate::CommunicationType)
    EnableRxAndTx = Self::ENABLE_RX_AND_TX,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`](crate::CommunicationType)
    EnableRxAndDisableTx = Self::ENABLE_RX_AND_DISABLE_TX,
    /// This value indicates that the reception of messages shall be disabled
    /// and the transmission of messages shall be enabled for the specified [`CommunicationType`](crate::CommunicationType)
    DisableRxAndEnableTx = Self::DISABLE_RX_AND_ENABLE_TX,
    /// This value indicates that the reception and transmission of messages
    /// shall be disabled for the specified [`CommunicationType`](crate::CommunicationType)
    DisableRxAndTx = Self::DISABLE_RX_AND_TX,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`](crate::CommunicationType)
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndDisableTxWithEnhancedAddressInfo =
        Self::ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFO,
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`](crate::CommunicationType)
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndTxWithEnhancedAddressInfo = Self::ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFO,
    /// These values are reserved by the ISO 14229-1 Specification
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u8),
    /// Values reserved for use by vehicle manufacturers
    #[cfg_attr(feature = "clap", clap(skip))]
    VehicleManufacturerSpecific(u8),
    /// Values reserved for use by system suppliers
    #[cfg_attr(feature = "clap", clap(skip))]
    SystemSupplierSpecific(u8),
}

impl CommunicationControlType {
    pub const ENABLE_RX_AND_TX: u8 = 0x00;
    pub const ENABLE_RX_AND_DISABLE_TX: u8 = 0x01;
    pub const DISABLE_RX_AND_ENABLE_TX: u8 = 0x02;
    pub const DISABLE_RX_AND_TX: u8 = 0x03;
    pub const ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFO: u8 = 0x04;
    pub const ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFO: u8 = 0x05;
    pub const ISO_RESERVED_START: u8 = 0x06;
    pub const ISO_RESERVED_END: u8 = 0x3F;
    pub const VEHICLE_MANUFACTURER_START: u8 = 0x40;
    pub const VEHICLE_MANUFACTURER_END: u8 = 0x5F;
    pub const SYSTEM_SUPPLIER_START: u8 = 0x60;
    pub const SYSTEM_SUPPLIER_END: u8 = 0x7E;
    pub const ISO_RESERVED_EXTENSION: u8 = 0x7F;
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
            CommunicationControlType::EnableRxAndTx => CommunicationControlType::ENABLE_RX_AND_TX,
            CommunicationControlType::EnableRxAndDisableTx => {
                CommunicationControlType::ENABLE_RX_AND_DISABLE_TX
            }
            CommunicationControlType::DisableRxAndEnableTx => {
                CommunicationControlType::DISABLE_RX_AND_ENABLE_TX
            }
            CommunicationControlType::DisableRxAndTx => CommunicationControlType::DISABLE_RX_AND_TX,
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo => {
                CommunicationControlType::ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFO
            }
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo => {
                CommunicationControlType::ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFO
            }
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
            Self::ENABLE_RX_AND_TX => Ok(Self::EnableRxAndTx),
            Self::ENABLE_RX_AND_DISABLE_TX => Ok(Self::EnableRxAndDisableTx),
            Self::DISABLE_RX_AND_ENABLE_TX => Ok(Self::DisableRxAndEnableTx),
            Self::DISABLE_RX_AND_TX => Ok(Self::DisableRxAndTx),
            Self::ENABLE_RX_AND_DISABLE_TX_WITH_ENHANCED_ADDRESS_INFO => {
                Ok(Self::EnableRxAndDisableTxWithEnhancedAddressInfo)
            }
            Self::ENABLE_RX_AND_TX_WITH_ENHANCED_ADDRESS_INFO => {
                Ok(Self::EnableRxAndTxWithEnhancedAddressInfo)
            }
            Self::ISO_RESERVED_START..=Self::ISO_RESERVED_END | Self::ISO_RESERVED_EXTENSION => {
                Ok(Self::ISOSAEReserved(value))
            }
            Self::VEHICLE_MANUFACTURER_START..=Self::VEHICLE_MANUFACTURER_END => {
                Ok(Self::VehicleManufacturerSpecific(value))
            }
            Self::SYSTEM_SUPPLIER_START..=Self::SYSTEM_SUPPLIER_END => {
                Ok(Self::SystemSupplierSpecific(value))
            }
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
