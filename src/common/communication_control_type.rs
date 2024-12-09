use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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

impl From<u8> for CommunicationControlType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::EnableRxAndTx,
            0x01 => Self::EnableRxAndDisableTx,
            0x02 => Self::DisableRxAndEnableTx,
            0x03 => Self::DisableRxAndTx,
            _ => panic!("Invalid communication enable: {value}",),
        }
    }
}
