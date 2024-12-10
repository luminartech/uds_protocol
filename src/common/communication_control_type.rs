use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::Error;

/// `CommunicationControlType` is used to specify the type of communication behavior to be modified
///
/// *Note*:
///
/// Conversions from `u8` to `DiagnosticSessionType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum CommunicationControlType {
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`]
    EnableRxAndTx,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`]
    EnableRxAndDisableTx,
    /// This value indicates that the reception of messages shall be disabled
    /// and the transmission of messages shall be enabled for the specified [`CommunicationType`]
    DisableRxAndEnableTx,
    /// This value indicates that the reception and transmission of messages
    /// shall be disabled for the specified [`CommunicationType`]
    DisableRxAndTx,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`]
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndDisableTxWithEnhancedAddressInfo,
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`]
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

impl From<CommunicationControlType> for u8 {
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
