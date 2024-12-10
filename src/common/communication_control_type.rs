use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum CommunicationControlType {
    EnableRxAndTx,
    EnableRxAndDisableTx,
    DisableRxAndEnableTx,
    DisableRxAndTx,
    EnableRxAndDisableTxWithEnhancedAddressInfo,
    EnableRxAndTxWithEnhancedAddressInfo,
    #[clap(skip)]
    ISOSAEReserved(u8),
    #[clap(skip)]
    VehicleManufacturerSpecific(u8),
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
            _ => panic!("Invalid communication enable: {value}",),
        }
    }
}
